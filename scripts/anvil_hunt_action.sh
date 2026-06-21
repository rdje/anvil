#!/usr/bin/env bash
# CI-PACKAGING-DISTRIBUTION.2b (decision 0022): the composite GitHub Action's
# entrypoint. A thin shim that (1) resolves an `anvil` binary — an explicit
# `anvil-bin` input, else the pinned release tarball for the runner OS/arch —
# (2) runs `anvil hunt` with the Action inputs mapped 1:1 onto its flags into a
# bundle directory, and (3) parses the `HuntReport` for the finding count and
# exposes it as a step output. It introduces NO private path: it drives the same
# `anvil hunt` CLI an interactive user or an MCP agent would (decision 0017).
#
# `anvil hunt` always exits 0 and prints the HuntReport JSON to stdout; the
# job's red/green is decided by action.yml from the `findings` output below.
#
# Inputs arrive as INPUT_* env vars (composite `run` steps do not auto-populate
# them — action.yml maps `inputs.*` into env explicitly).
set -euo pipefail

ANVIL_BIN="${INPUT_ANVIL_BIN:-}"
VERSION="${INPUT_ANVIL_VERSION:-}"
TOOLS="${INPUT_TOOLS:-verilator,yosys}"
SEED="${INPUT_SEED:-42}"
SEEDS="${INPUT_SEEDS:-64}"
PROFILE="${INPUT_PROFILE:-}"
CONFIG="${INPUT_CONFIG:-}"
YOSYS_MODE="${INPUT_YOSYS_MODE:-without-abc}"
DIFF_SIM="${INPUT_DIFF_SIM:-false}"
DIVERGENCE="${INPUT_DIVERGENCE:-false}"
BUDGET="${INPUT_BUDGET:-200}"
NO_MINIMIZE="${INPUT_NO_MINIMIZE:-false}"
OUT="${INPUT_OUT:-}"
if [ -z "$OUT" ]; then OUT="${GITHUB_WORKSPACE:-$PWD}/anvil-hunt-bundles"; fi

# --- 1. Resolve the anvil binary --------------------------------------------
if [ -n "$ANVIL_BIN" ]; then
  anvil="$ANVIL_BIN"
else
  # Default the version to the ref the Action was pinned to (e.g. `v0.1.0`),
  # so `uses: <owner>/anvil@v0.1.0` pins the binary to the matching release.
  [ -n "$VERSION" ] || VERSION="${GITHUB_ACTION_REF:-}"
  if [ -z "$VERSION" ]; then
    echo "::error::anvil-version is empty and GITHUB_ACTION_REF is unset — pin the Action to a release tag or pass anvil-bin." >&2
    exit 1
  fi
  repo="${GITHUB_ACTION_REPOSITORY:-}"
  if [ -z "$repo" ]; then
    echo "::error::GITHUB_ACTION_REPOSITORY is unset — cannot locate the release to download. Pass anvil-bin for a local build." >&2
    exit 1
  fi
  case "${RUNNER_OS:-}" in
    Linux)   os="unknown-linux-gnu"; ext="tar.gz" ;;
    macOS)   os="apple-darwin";      ext="tar.gz" ;;
    Windows) os="pc-windows-msvc";   ext="zip" ;;
    *) echo "::error::unsupported RUNNER_OS '${RUNNER_OS:-}'." >&2; exit 1 ;;
  esac
  case "${RUNNER_ARCH:-}" in
    X64)   arch="x86_64" ;;
    ARM64) arch="aarch64" ;;
    *) echo "::error::unsupported RUNNER_ARCH '${RUNNER_ARCH:-}'." >&2; exit 1 ;;
  esac
  target="${arch}-${os}"
  archive="anvil-${VERSION}-${target}.${ext}"
  url="https://github.com/${repo}/releases/download/${VERSION}/${archive}"
  workdir="$(mktemp -d)"
  echo "Downloading ${url}"
  curl -fsSL "$url" -o "${workdir}/${archive}"
  if [ "$ext" = "zip" ]; then
    unzip -q "${workdir}/${archive}" -d "$workdir"
    anvil="${workdir}/anvil.exe"
  else
    tar -xzf "${workdir}/${archive}" -C "$workdir"
    anvil="${workdir}/anvil"
  fi
  chmod +x "$anvil" 2>/dev/null || true
fi

echo "Using anvil binary: ${anvil}"

# --- 2. Run `anvil hunt` with the mapped inputs -----------------------------
args=(hunt --seed "$SEED" --seeds "$SEEDS" --yosys-mode "$YOSYS_MODE" --budget "$BUDGET" --out "$OUT")
[ -n "$TOOLS" ]               && args+=(--tools "$TOOLS")
[ -n "$PROFILE" ]             && args+=(--profile "$PROFILE")
[ -n "$CONFIG" ]              && args+=(--config "$CONFIG")
[ "$DIFF_SIM" = "true" ]      && args+=(--diff-sim)
[ "$DIVERGENCE" = "true" ]    && args+=(--divergence)
[ "$NO_MINIMIZE" = "true" ]   && args+=(--no-minimize)

mkdir -p "$OUT"
report="${OUT}/hunt-report.json"
echo "Running: anvil ${args[*]}"
"$anvil" "${args[@]}" | tee "$report"

# --- 3. Parse the HuntReport for the finding count --------------------------
# python3 is preinstalled on every GitHub-hosted runner (no jq dependency).
findings="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["summary"]["n_failures"])' "$report")"

{
  echo "findings=${findings}"
  echo "report=${report}"
  echo "bundle-dir=${OUT}"
} >> "${GITHUB_OUTPUT:-/dev/stdout}"

last=$(( SEED + SEEDS - 1 ))
echo "ANVIL hunt: ${findings} finding(s) across seeds ${SEED}..${last} on tools [${TOOLS}]."
