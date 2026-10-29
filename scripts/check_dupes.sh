#!/usr/bin/env bash
# Detect duplicate fn names in Rust files (sign of a bad paste).
set -euo pipefail
find crates -name "*.rs" -type f | while read -r f; do
    dupes=$(grep -oE "^\s*(pub )?fn [a-z_0-9]+" "$f" | awk '{print $NF}' | sort | uniq -d)
    if [ -n "$dupes" ]; then
        echo "[$f]"
        echo "$dupes" | sed 's/^/  /'
    fi
done
