# Catfight Bonsai Tournament

*A practical guide to growing digital forests*

---

## The Vision

Use clood's catfight system to generate hundreds of bonsai trees across multiple machines, find the most beautiful ones, and transform them into plottable SVGs.

This serves two purposes:
1. **Art**: Discover aesthetically pleasing bonsais through parameter exploration
2. **Testing**: Stress-test the cross-platform catfight infrastructure

---

## Phase 1: The Forest (Generation)

### Parameter Space

cbonsai has two key parameters:
- `-L` (life): Controls tree height/complexity (10-200)
- `-M` (multiplier): Controls branch density (2-20)

The interaction creates distinct tree types:

| Life | Multiplier | Result |
|------|------------|--------|
| Low | Low | Tiny seedling |
| Low | High | Short, bushy shrub |
| High | Low | Tall, spindly tree |
| High | High | Dense ancient growth |
| Balanced | Balanced | Classic bonsai |

### Generation Script

```bash
#!/bin/bash
# bonsai_forest.sh - Generate a forest of bonsai trees

OUTPUT_DIR="${1:-/tmp/bonsai_forest}"
mkdir -p "$OUTPUT_DIR"

# Parameter ranges
LIVES=(10 20 32 50 80 100)
MULTS=(2 3 5 8 12)

count=0
for life in "${LIVES[@]}"; do
  for mult in "${MULTS[@]}"; do
    # Generate 3 trees per parameter combo (different random seeds)
    for seed in 1 2 3; do
      filename="bonsai_L${life}_M${mult}_${seed}.txt"
      cbonsai -p -L $life -M $mult > "$OUTPUT_DIR/$filename"
      ((count++))
    done
  done
done

echo "Generated $count bonsai trees in $OUTPUT_DIR"
```

This generates 90 trees (6 life values × 5 multipliers × 3 seeds).

---

## Phase 2: The Tournament (Catfight)

### Rating Prompt

```markdown
# Bonsai Beauty Contest

You are judging ASCII bonsai trees for aesthetic quality.

## Criteria
1. **Balance**: Is the tree visually balanced, not lopsided?
2. **Negative Space**: Is there breathing room, or is it too dense?
3. **Trunk/Branch Ratio**: Is the trunk visible but not dominant?
4. **Organic Feel**: Does it look like a natural tree, not random noise?
5. **Plotter Suitability**: Would this look good when pen-plotted?

## Rating Scale
- 1-2: Poor (unbalanced, too sparse, or too chaotic)
- 3-4: Fair (recognizable but flawed)
- 5-6: Good (pleasant, minor issues)
- 7-8: Very Good (balanced, organic, plottable)
- 9-10: Excellent (would frame and hang on wall)

## Trees to Rate

{{BONSAI_CONTENT}}

## Output Format

For each tree, provide:
- Rating (1-10)
- One sentence explanation
- Whether you'd recommend it for plotting (yes/no)
```

### Running the Tournament

```bash
# From clood-cli
clood catfight \
  -f bonsai_rating_prompt.md \
  --models "deepseek-coder:6.7b,mistral:7b,qwen2.5-coder:7b" \
  --json \
  -o battles/bonsai_tournament.json
```

### Cross-Kitchen Mode

```bash
# Run the same forest on both kitchens
clood catfight \
  --hosts "ubuntu25,mac-mini" \
  -f bonsai_rating_prompt.md \
  --json \
  -o battles/bonsai_cross_kitchen.json
```

This tests:
- Network latency between kitchens
- Model performance on different hardware
- Consistency of aesthetic judgments across instances

---

## Phase 3: The Selection (Curation)

### Finding Winners

After the tournament, parse the JSON results:

```python
#!/usr/bin/env python3
# find_winners.py

import json
import sys
from pathlib import Path

def find_winners(results_path, threshold=7):
    with open(results_path) as f:
        results = json.load(f)

    winners = []
    for entry in results.get("entries", []):
        # Parse ratings from all models
        ratings = extract_ratings(entry)
        avg_rating = sum(ratings) / len(ratings) if ratings else 0

        if avg_rating >= threshold:
            winners.append({
                "file": entry["file"],
                "avg_rating": avg_rating,
                "ratings": ratings,
                "consensus": all(r >= threshold for r in ratings)
            })

    # Sort by average rating
    winners.sort(key=lambda x: x["avg_rating"], reverse=True)

    print(f"Found {len(winners)} winners (threshold: {threshold})")
    for w in winners[:10]:
        consensus = "CONSENSUS" if w["consensus"] else ""
        print(f"  {w['avg_rating']:.1f} - {w['file']} {consensus}")

if __name__ == "__main__":
    find_winners(sys.argv[1])
```

### Manual Review

The top-rated trees should be reviewed by human eyes:

```bash
# View top candidates
for file in $(cat winners.txt | head -10); do
  echo "=== $file ==="
  cat $file
  echo ""
  read -p "Keep? (y/n) " choice
done
```

---

## Phase 4: The Transformation (ascii2svg)

### Convert Winners to SVG

```bash
# Once ascii2svg is implemented (Issue #11)
for winner in winners/*.txt; do
  base=$(basename "$winner" .txt)

  # Try multiple fonts
  rat-king ascii2svg "$winner" -f futural -o "svg/${base}_futural.svg"
  rat-king ascii2svg "$winner" -f scripts -o "svg/${base}_scripts.svg"
  rat-king ascii2svg "$winner" -f EMSDelight -o "svg/${base}_delight.svg"
done
```

### Add Spiral Background

```bash
# Get dimensions from SVG
dims=$(grep 'viewBox' svg/winner_futural.svg | grep -oP '\d+ \d+$')
width=$(echo $dims | cut -d' ' -f1)
height=$(echo $dims | cut -d' ' -f2)

# Generate matching spiral background
rat-king fill --rect "${width}x${height}" -p spiral -s 15 -o background.svg

# Combine (using vpype)
vpype read background.svg read svg/winner_futural.svg write combined.svg
```

---

## Phase 5: The Plotting

### Prepare for Plotter

```bash
# Optimize paths
vpype read combined.svg \
  linesort \
  linemerge \
  write --device hp7475a output.hpgl

# Or for gcode
vpype read combined.svg \
  linesort \
  linemerge \
  write output.gcode
```

### Layer-by-Layer Plotting

If using color layers:

```bash
# Plot background first (single color)
vpype read background.svg write bg.hpgl

# Then each bonsai layer
vpype read svg/winner.svg \
  read --layer 1 \
  write trunk.hpgl

vpype read svg/winner.svg \
  read --layer 2 \
  write foliage.hpgl
```

---

## The Archive

Vincenzo, Keeper of the Fulton Street Archives, has requested that the most beautiful bonsais be preserved.

Store winners in:
```
lore/bonsai_archive/
├── 2025-12/
│   ├── L32_M5_seed42.txt      # ASCII original
│   ├── L32_M5_seed42.svg      # Converted SVG
│   ├── L32_M5_seed42.json     # Catfight ratings
│   └── L32_M5_seed42.jpg      # Photo of plot (if plotted)
```

---

## Vincenzo's Recommendation

From the Letter:

> The most beautiful bonsai came from `-L 32 -M 5`.

Start your experiments there:

```bash
cbonsai -p -L 32 -M 5
```

Then explore the neighborhood:
- `-L 28 -M 5` (slightly smaller)
- `-L 36 -M 5` (slightly larger)
- `-L 32 -M 4` (less dense)
- `-L 32 -M 6` (more dense)

---

## Success Metrics

The tournament is successful if:

1. **Art**: At least 5 bonsais are beautiful enough to plot
2. **Testing**: Both kitchens complete the workload without failures
3. **Consensus**: Models agree on what makes a good bonsai (>60% rating alignment)
4. **Pipeline**: ascii2svg → vpype → plotter works end-to-end

---

*"The catfight is not a test workload. It is a garden that grows itself."*
*— Vincenzo*
