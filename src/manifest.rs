//! Streaming `manifest.json` writer (`WORKLOAD-MEMORY-SAFETY.2`).
//!
//! The directory-output path (`--out DIR`) used to build a
//! `Vec<serde_json::Value>` holding **every** per-artifact metadata
//! object for the whole run, then `serde_json::to_string_pretty` the
//! lot in one shot. That makes peak memory grow O(`--count`): a
//! million-module run accumulates a million metrics objects in RAM
//! before a single byte is written — enough to drive a RAM-limited
//! host toward the danger/reboot zone.
//!
//! [`write_streamed_manifest`] instead writes the array **element by
//! element** to a [`std::io::Write`] (a `BufWriter` over the file),
//! so only one element Value is live at a time and peak metadata
//! memory is O(1) in the artifact count.
//!
//! ## Byte-identical contract
//!
//! The bytes produced are **byte-for-byte identical** to the previous
//! `serde_json::to_string_pretty` of the fully-assembled object — the
//! reproducibility contract (`book/src/knobs.md`) is non-negotiable and
//! `manifest.json` is user-visible. Two devices guarantee this:
//!
//! 1. The surrounding object framing (key order — serde_json sorts
//!    object keys since the crate does not enable `preserve_order` —
//!    the `seed`/`config` rendering, commas, indentation) is taken
//!    *directly from serde* by serialising the same object with the
//!    array key bound to a unique placeholder string and splitting the
//!    result around that placeholder. Nothing about the framing is
//!    hand-rolled.
//! 2. Each array element is rendered by `serde_json::to_string_pretty`
//!    and then re-indented by the constant base offset of its nesting
//!    depth. Pretty-print indentation is purely a function of depth, so
//!    prefixing every line of a standalone-pretty element with the base
//!    indent reproduces exactly the bytes serde would emit for that
//!    element nested in the array.
//!
//! [`streamed_matches_reference`] proves (1)+(2) against serde itself.

use serde_json::Value;
use std::io::{self, Write};

/// Unique sentinel that cannot collide with real manifest content. It
/// is inserted as the array key's value purely so we can locate where
/// serde placed that key in the sorted-key framing, then replace it
/// with the streamed array body.
const ARRAY_PLACEHOLDER: &str = "__ANVIL_STREAM_ARRAY_PLACEHOLDER__";

/// Write a top-level manifest object `{ <scalars…>, "<array_key>": [ … ] }`
/// to `w`, streaming the array elements so peak memory is O(1) in the
/// number of elements while the bytes stay identical to
/// `serde_json::to_string_pretty` of the fully-assembled object.
///
/// * `scalars` — the non-array top-level fields (e.g. `seed`, `config`).
///   Must **not** already contain `array_key`.
/// * `array_key` — the streamed array's key (`"modules"` or `"designs"`).
/// * `elements` — an iterator yielding each element lazily as
///   `io::Result<Value>`; the caller's closure does the per-element work
///   (generate, write the `.sv`, build the metadata) so generation stays
///   bounded too. An `Err` aborts the write and propagates.
///
/// The writer is flushed before returning.
pub fn write_streamed_manifest<W, I>(
    mut w: W,
    scalars: &serde_json::Map<String, Value>,
    array_key: &str,
    elements: I,
) -> io::Result<()>
where
    W: Write,
    I: IntoIterator<Item = io::Result<Value>>,
{
    debug_assert!(
        !scalars.contains_key(array_key),
        "scalars must not already contain the streamed array key",
    );

    // Derive the exact surrounding framing from serde by serialising the
    // object with the array key bound to a locatable placeholder string.
    let mut framing = scalars.clone();
    framing.insert(
        array_key.to_string(),
        Value::String(ARRAY_PLACEHOLDER.to_string()),
    );
    let framed = serde_json::to_string_pretty(&Value::Object(framing)).map_err(io_err)?;

    // The placeholder renders as a quoted JSON string on the key line:
    //   `  "<array_key>": "__ANVIL_STREAM_ARRAY_PLACEHOLDER__"`.
    let token = format!("\"{ARRAY_PLACEHOLDER}\"");
    let pos = framed
        .find(&token)
        .expect("array placeholder must be present in the framing");
    let head = &framed[..pos]; // up to and including `"<array_key>": `
    let tail = &framed[pos + token.len()..]; // from just after the placeholder

    // Top-level keys sit at one indent step (2 spaces); array elements at
    // two steps (4 spaces). These are serde_json pretty-print invariants
    // for an array that is a direct child of the root object, which the
    // manifest array always is. `streamed_matches_reference` guards them.
    const ELEM_INDENT: &str = "    "; // 4 spaces
    const CLOSE_INDENT: &str = "  "; // 2 spaces

    w.write_all(head.as_bytes())?;
    w.write_all(b"[")?;

    let mut any = false;
    for elem in elements {
        let elem = elem?;
        w.write_all(if any { b",\n" } else { b"\n" })?;
        any = true;
        let pretty = serde_json::to_string_pretty(&elem).map_err(io_err)?;
        w.write_all(ELEM_INDENT.as_bytes())?;
        // Re-indent every interior line by the element base indent.
        w.write_all(pretty.replace('\n', "\n    ").as_bytes())?;
    }

    if any {
        w.write_all(b"\n")?;
        w.write_all(CLOSE_INDENT.as_bytes())?;
        w.write_all(b"]")?;
    } else {
        w.write_all(b"]")?;
    }
    w.write_all(tail.as_bytes())?;
    w.flush()
}

fn io_err(e: serde_json::Error) -> io::Error {
    io::Error::other(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalars() -> serde_json::Map<String, Value> {
        // Mirror the real manifest's scalar fields: a `seed` (sorts after
        // the array key) and a nested `config`-like object (sorts before).
        let mut m = serde_json::Map::new();
        m.insert("seed".to_string(), serde_json::json!(42));
        m.insert(
            "config".to_string(),
            serde_json::json!({
                "max_depth": 6,
                "flop_prob": 0.15,
                "nested": { "a": [1, 2, 3], "b": "x/y" },
                "list": [true, false],
            }),
        );
        m
    }

    fn module_elem(i: u64) -> Value {
        serde_json::json!({
            "file": format!("mod_42_{i:04}.sv"),
            "name": format!("mod_{i}"),
            "metrics": { "nodes": i * 7 + 1, "depth": 3, "frac": 0.5, "label": "a\"b" },
        })
    }

    /// The streamed bytes must equal `to_string_pretty` of the
    /// fully-assembled object for every element count, including 0
    /// (empty array `[]`) and the key-after-array case (`seed` sorts
    /// after `modules`).
    fn reference(s: &serde_json::Map<String, Value>, key: &str, elems: &[Value]) -> String {
        let mut o = s.clone();
        o.insert(key.to_string(), Value::Array(elems.to_vec()));
        serde_json::to_string_pretty(&Value::Object(o)).unwrap()
    }

    #[test]
    fn streamed_matches_reference() {
        let s = scalars();
        for n in [0usize, 1, 2, 5, 17] {
            let elems: Vec<Value> = (0..n as u64).map(module_elem).collect();
            let mut buf: Vec<u8> = Vec::new();
            write_streamed_manifest(&mut buf, &s, "modules", elems.iter().cloned().map(Ok))
                .unwrap();
            let got = String::from_utf8(buf).unwrap();
            assert_eq!(got, reference(&s, "modules", &elems), "mismatch at n={n}");
        }
    }

    /// The same guarantee for the hierarchical lane's `designs` array,
    /// whose elements carry a nested `modules` sub-array (deeper nesting,
    /// exercising the re-indentation on a multi-level element).
    #[test]
    fn streamed_matches_reference_for_designs() {
        let s = scalars();
        let design = |i: u64| {
            serde_json::json!({
                "index": i,
                "top": format!("top_{i}"),
                "metrics": { "modules": 3, "depth": i },
                "modules": [ module_elem(i * 2), module_elem(i * 2 + 1) ],
            })
        };
        for n in [0usize, 1, 3] {
            let elems: Vec<Value> = (0..n as u64).map(design).collect();
            let mut buf: Vec<u8> = Vec::new();
            write_streamed_manifest(&mut buf, &s, "designs", elems.iter().cloned().map(Ok))
                .unwrap();
            let got = String::from_utf8(buf).unwrap();
            assert_eq!(got, reference(&s, "designs", &elems), "mismatch at n={n}");
        }
    }

    /// An element-producing error aborts the write and propagates.
    #[test]
    fn propagates_element_error() {
        let s = scalars();
        let elems = vec![Ok(module_elem(0)), Err(io::Error::other("boom"))];
        let mut buf: Vec<u8> = Vec::new();
        let r = write_streamed_manifest(&mut buf, &s, "modules", elems);
        assert!(r.is_err());
    }
}
