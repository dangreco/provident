#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
#
# Final `CI` job: report job results and merged coverage to the step summary, write the merged
# LCOV report, and fail if any job failed or was cancelled.
#
# Usage: ci-summary.sh <lcov-dir> <merged-lcov-out>
# Env:   NEEDS                `toJSON(needs)` of the summary job
#        GITHUB_STEP_SUMMARY  markdown output (defaults to stdout)
#        GITHUB_WORKSPACE     stripped from LCOV paths (defaults to $PWD)
#
# Coverage is the union across all reports: a line counts as covered if any job (unit or e2e,
# async or blocking) executed it. Lines compiled in only one mode are counted once.

set -euo pipefail

lcov_dir=$1
merged=$2
summary=${GITHUB_STEP_SUMMARY:-/dev/stdout}
root=${GITHUB_WORKSPACE:-$PWD}/

{
  echo "## CI"
  echo
  echo "| Job | Result |"
  echo "| --- | --- |"
  jq -r 'to_entries[] | "| \(.key) | \(.value.result) |"' <<<"$NEEDS"
  echo
} >>"$summary"

mapfile -t reports < <(find "$lcov_dir" -name '*.lcov' -type f 2>/dev/null | sort)

if ((${#reports[@]} == 0)); then
  printf '### Coverage\n\nNo coverage reports were produced.\n' >>"$summary"
else
  # file<TAB>line<TAB>hits for every DA record, keeping the max hits per line.
  awk -v root="$root" '
    /^SF:/ { file = substr($0, 4); if (index(file, root) == 1) file = substr(file, length(root) + 1); next }
    /^DA:/ { split(substr($0, 4), da, ","); print file "\t" da[1] "\t" da[2] }
  ' "${reports[@]}" |
    sort -t$'\t' -k1,1 -k2,2n -k3,3nr |
    awk -F'\t' '!seen[$1 FS $2]++' >"$merged.tsv"

  # Merged LCOV (relative paths).
  awk -F'\t' '
    $1 != file { if (file != "") printf "LF:%d\nLH:%d\nend_of_record\n", lf, lh; file = $1; lf = lh = 0; print "SF:" file }
    { print "DA:" $2 "," $3; lf++; if ($3 > 0) lh++ }
    END { if (file != "") printf "LF:%d\nLH:%d\nend_of_record\n", lf, lh }
  ' "$merged.tsv" >"$merged"

  {
    echo "### Coverage"
    echo
    # Artifact name = parent directory (download-artifact extracts each into its own dir).
    names=$(for r in "${reports[@]}"; do basename "$(dirname "$r")"; done | paste -sd, - | sed 's/,/, /g')
    echo "Line coverage merged from ${#reports[@]} report(s): $names"
    echo
    echo "| File | Lines | Covered | % |"
    echo "| --- | ---: | ---: | ---: |"
    awk -F'\t' '
      { lines[$1]++; if ($3 > 0) hit[$1]++; total++; if ($3 > 0) covered++ }
      END {
        for (f in lines) printf "| `%s` | %d | %d | %.1f |\n", f, lines[f], hit[f], 100 * hit[f] / lines[f] | "sort"
        close("sort")
        printf "| **Total** | **%d** | **%d** | **%.1f** |\n", total, covered, total ? 100 * covered / total : 0
      }
    ' "$merged.tsv"
  } >>"$summary"
  rm "$merged.tsv"
fi

failed=$(jq -r '[to_entries[] | select(.value.result == "failure" or .value.result == "cancelled") | .key] | join(", ")' <<<"$NEEDS")
if [[ -n $failed ]]; then
  echo "::error::Failed or cancelled: $failed"
  exit 1
fi
