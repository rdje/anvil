#!/usr/bin/env bash
# scripts/ram_guard.sh — run a command under a RAM watchdog.
#
# Aborts the wrapped command (and its process group) BEFORE the host
# enters the danger zone, so heavy cargo builds/tests/tool_matrix sweeps
# cannot drive this RAM-limited host toward the ~95%-used level that
# triggers a reboot. The default threshold (88%) sits below the 90%
# danger line recorded in the project's resource policy
# (docs/decisions/0003-resource-safe-validation.md).
#
# Usage:
#   scripts/ram_guard.sh [--threshold PCT] [--interval SEC] -- CMD [ARGS...]
#
# Options:
#   --threshold PCT   abort when used RAM% >= PCT   (default 88)
#   --interval SEC    poll period in seconds        (default 3)
#
# Exit status:
#   the wrapped command's own status if it finishes on its own;
#   99 if the guard aborted it for crossing the RAM threshold;
#   2  on usage error.
#
# RAM reading: macOS uses `memory_pressure` ("free percentage", so
# used = 100 - free); Linux uses /proc/meminfo MemAvailable. An
# unreadable sample is treated as "no abort" so a probe hiccup never
# kills a healthy job.
set -uo pipefail

threshold=88
interval=3

usage() { sed -n '2,24p' "$0" >&2; exit 2; }

while [ $# -gt 0 ]; do
  case "$1" in
    --threshold) threshold="${2:?--threshold needs a value}"; shift 2;;
    --interval)  interval="${2:?--interval needs a value}";  shift 2;;
    --) shift; break;;
    -h|--help) usage;;
    *) echo "ram_guard: unknown arg: $1" >&2; usage;;
  esac
done
[ $# -ge 1 ] || { echo "ram_guard: no command given (use: ... -- CMD)" >&2; usage; }

used_pct() {
  case "$(uname -s)" in
    Darwin)
      local free
      free="$(memory_pressure 2>/dev/null \
        | awk -F': ' '/free percentage/{gsub(/[ %]/,"",$2); print $2; exit}')"
      [ -n "$free" ] && awk -v f="$free" 'BEGIN{printf "%d", 100 - f}'
      ;;
    Linux)
      awk '/^MemTotal:/{t=$2} /^MemAvailable:/{a=$2}
           END{ if (t > 0) printf "%d", (t - a) * 100 / t }' /proc/meminfo
      ;;
  esac
}

# Run the command in its own process group so the whole tree can be signalled.
set -m
"$@" &
cmd_pid=$!

aborted=0
while kill -0 "$cmd_pid" 2>/dev/null; do
  u="$(used_pct)"
  if [ -n "${u:-}" ] && [ "$u" -ge "$threshold" ]; then
    echo "ram_guard: ABORT — used RAM ${u}% >= threshold ${threshold}%; stopping PID ${cmd_pid}" >&2
    kill -TERM "-${cmd_pid}" 2>/dev/null || kill -TERM "$cmd_pid" 2>/dev/null
    sleep 2
    kill -KILL "-${cmd_pid}" 2>/dev/null || kill -KILL "$cmd_pid" 2>/dev/null
    aborted=1
    break
  fi
  sleep "$interval"
done

if [ "$aborted" -eq 1 ]; then
  wait "$cmd_pid" 2>/dev/null
  exit 99
fi
wait "$cmd_pid"
exit $?
