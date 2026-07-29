# `docs/evidence/` — committed closure-evidence digests

The tracked layer that makes a closure claim re-verifiable after the machine
that produced it is gone. Contract: decision
[`0030`](../decisions/0030-durable-closure-evidence-citations.md) and its
`2026-07-30` amendment. Gated by `scripts/check_evidence_citations.sh`
(`EVIDENCE-CITATIONS` in `scripts/check_doctrines.sh`).

## Why this exists

Every numbered-phase exit and every opt-in surface gate in ANVIL was justified
by a *banked artifact* — a `tool_matrix_report.json` or a parity corpus — cited
by path. At the `2026-07-25` audit that opened `EVIDENCE-BANK-DURABILITY`,
**every one of those artifacts was gone**: they lived under the OS temp dir,
which is purged. 73 citations pointed at nothing.

The corpus was never the thing worth keeping — a `--phase4-hierarchy-gate` bank
is 840 designs and has no business in git. What must survive is the handful of
**claim-bearing numbers**: how many units ran, what each tool said,
whether `coverage_gaps` was empty, and the exact command that reproduces it. A
few KB per bank, tracked, greppable, diffable, clone-portable.

## What a digest is

One file per bank: `docs/evidence/<bank-name>.md`, where `<bank-name>` is the
run's directory basename (`anvil-case-mux-if-gate-r2`). Generated, not written
by hand:

```bash
scripts/evidence_digest.sh <report.json> <bank-name> \
    --leaf <TREE.LEAF> --claim '<what this backs>' [--command '<cmd>']
```

The check enforces these fields (see `check_evidence_citations.sh` leg 2):

| Field | Rule |
| --- | --- |
| `bank` | must equal the filename stem — the digest cannot be misfiled |
| `claim` | non-empty; what closure or gate this evidence backs |
| `owning_leaf` | the task-tree leaf that owns the claim |
| `commit` | 7–40 hex; **the commit the run executed at** |
| `date` | `YYYY-MM-DD` |
| `command` | the exact re-runnable oracle — mechanism (i) of `0030`, embedded |
| `report_sha256` | 64 hex, of the source `tool_matrix_report.json` |
| `coverage_gaps` | present; `[]` is what backs a closure claim |

`commit` is load-bearing and easy to under-value: a later commit may
legitimately produce different numbers, so a digest without the commit it ran at
is not re-verifiable — it is just a number that once was true.

## Where banks are written

Under `.cache/anvil-sandbox/<bank-name>/` — on the project volume (decision
`0031`), gitignored, resolved by `src/paths.rs`. **The digest is the only
tracked artifact.** Never write a bank to an OS temp dir; never commit a corpus.

## The three buckets

`INVENTORY.md` carries the frozen classification. Every `anvil-<name>` token in
the scanned document set is classified exactly once:

1. **digest-backed** — a file here. The forward path; grows without limit.
2. **grandfathered** — `INVENTORY.md` §1: a pre-`0030` bank whose artifact is
   gone. **Frozen**, pinned by count and SHA-256 in the check.
3. **not evidence** — `INVENTORY.md` §2: binaries, directories, Action inputs,
   negative-control fixtures, prose. Grows under review.

Anything unclassified fails the check. It fails **closed** — an author who adds
a new banked claim cannot simply omit the digest.

## Why the check does not just pattern-match

Decision `0030` specified keying on a bare `/tmp/anvil-*` path. That
discriminator no longer exists — `VOLUME-DATA-LOCALITY.5` stripped every `/tmp/`
prefix — and what remains is a bare `anvil-<name>` token, a shape ANVIL also
uses for binaries (`anvil-mcp`), directories (`anvil-sandbox`), Action inputs
(`anvil-bin`) and English prose (`anvil-emitted`). No lexical rule separates
`anvil-cf-sweep` (a bank) from `anvil-hunt-bundles` (not one).

So the check does not guess. It requires every token to be *classified*, and
treats an unknown token as a breach. A heuristic would have to be wrong in one
of two directions — silently ignoring a real uncited bank, or crying wolf on
prose — and a gate that cries wolf gets disabled.

## Adding a new bank (the whole workflow)

```bash
# 1. run the gate, on-volume
cargo run --release --bin tool_matrix -- --<surface>-gate --yosys-mode both \
    --out .cache/anvil-sandbox/anvil-<surface>-gate-r1

# 2. derive the digest
scripts/evidence_digest.sh \
    .cache/anvil-sandbox/anvil-<surface>-gate-r1/tool_matrix_report.json \
    anvil-<surface>-gate-r1 --leaf <TREE.LEAF> --claim '<what it backs>'

# 3. review the derived `command`, then cite the bank name in the live doc
# 4. commit the digest with the claim  (the check now admits the citation)
```
