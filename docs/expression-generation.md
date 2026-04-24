# Expression Generation — Biases & Tradeoffs

Reference for how random and mutated expression trees are built, and what biases the code imposes on that distribution. Written to be read alongside `src/genome/op.rs`, `src/genome/node.rs`, `src/evolution.rs`, and `src/config.rs`.

## Why this document exists

Naive genetic programming for images produces *buzz* — sin-noise, hashed fractals, high-frequency ringing that looks the same every generation. The math is right, but the prior is wrong. Selection pressure alone can't fix a bad prior: every child is drawn from the same distribution as its parents, so if that distribution favors noise, every offspring drifts back toward noise.

This codebase therefore **puts deliberate fingers on the scale** during random generation and mutation to bias genomes toward lower spatial frequency, smoother compositions, and shallower trees — without eliminating the complex end of the range entirely.

See Karl Sims, *Artificial Evolution for Computer Graphics* (SIGGRAPH 1991) for the origin of the expression-tree-GP-for-images idea; Penousal Machado's **NEvAr** papers and *Evolutionary Art and Computers* (Todd & Latham 1992) cover complexity/aesthetic control in more depth.

## The evaluation pipeline

Every pixel runs through a 4-stage pipeline:

```
(nx, ny) normalized to [-1, 1]
        │
        ▼   3 spatial genomes (one per channel)
   c0_raw, c1_raw, c2_raw  ∈ ℝ
        │   fract wrap → [0, 1)
        ▼   3 palette-remap genomes (t = c_raw)
   c0, c1, c2  ∈ [0, 1)
        │
        ▼   channels_to_rgb[color_model]   ← 0=HSV 1=RGB 2=HSL 3=CMY 4=YUV
   (r, g, b)  ∈ [0, 1]
```

- **Coordinate space**: normalized to `[-1, 1]` (not screen pixels). A sine of raw `X` completes ~⅓ of a cycle across the tile — low-frequency by default.
- **Two-stage trees per channel**: a *spatial* tree sees `(x, y)` and produces a raw channel; a *palette* tree sees a scalar `t` (the fract'd raw channel) and reshapes it. The split lets evolution tune color independently from geometry.
- **Wrapping with `fract`**: applied at the boundary between stages. This keeps the pipeline bounded but introduces one hard discontinuity per stage — unavoidable, and part of why we don't want `fract` also appearing inside trees (see the weights below).
- **Color model** is a per-individual `u32` that selects the final 3-channel → RGB conversion. Evolvable, rarely mutated (see `config::COLOR_MODEL_MUTATION_PROB`).

---

## The fingers on the scale

### 1. Operator weights (`src/genome/op.rs`)

`weighted_choice` picks from the op registry using per-op `weight` fields. The weights form a **low-frequency prior**: at every non-terminal node, smooth composing ops are more likely than discontinuity ops.

| Category | Ops | Weight | Rationale |
|---|---|---|---|
| **Smooth composers** (boosted) | `Sin`, `Cos`, `Tanh`, `Mix`, `Smoothstep` | 1.5–2.0 | Bounded outputs, smooth derivatives; `Mix` at 2.0 because it's a gentle blend. |
| **Linear composers** (boosted) | `Add`, `Sub` | 1.5 | Superpose features without frequency multiplication. |
| **Raw coordinate terminals** (boosted) | `X`, `Y`, `MirrorX`, `MirrorY` | 1.5 | Give shallow trees a real chance of bottoming out in a smooth input. |
| **Neutral** | `Mul`, `Min`, `Max`, `Abs`, `Sqrt`, `Atan`, `Atan2`, `Negate`, `Const`, `PaletteT`, `ScaledX`, `ScaledY` | 1.0 | Needed but not pushed. `Mul` is kept flat despite being a frequency multiplier — dropping it would gut expressiveness. |
| **Mild penalty** | `Log`, `Clamp→1.2`, `Invert`, `Asin`, `Acos`, `TriWave`, `SinFold`, `Chebyshev`, `Manhattan`, `Worley`, `Ridged`, `Billow` | 0.5 | Still useful but not dominant. Cellular-distance ops (`Worley`, `Chebyshev`, `Manhattan`) produce visible cell boundaries; `Ridged`/`Billow` create sharp ridges and valleys. |
| **Noise defaults** | `ValueNoise`, `FBM`, `Turbulence`, `SimplexNoise`, `DomainWarp` | 0.8 | Below 1.0 so noise no longer dominates populations, but not so low that it disappears. |
| **Heavy penalty** | `Fract`, `Mod`, `Floor`, `Sign`, `Step`, `Tan`, `Div`, `Pow`, `Exp`, `Sinh`, `Cosh`, `Reciprocal` | 0.3 | Sources of infinite-frequency content (discontinuities) or unbounded blow-up (divide/exp). Selection rarely picks these, but they're still reachable. |

**Applied in two places**: fresh random nodes (`Node::random_with_depth`) and expression-swap mutation (`random_op_same_arity`). Subtree-replacement mutation (`mutate_subtree`) preserves the original op at that position, so it doesn't need re-weighting.

---

### 2. Tree depth distribution (`evolution::sample_tree_depth`)

Depth 1..=9 is sampled uniformly; 10, 11, 12, 13 halve in weight each step:

```
depths 1..=9  →  weight 1.0 each
depth 10      →  weight 0.5
depth 11      →  weight 0.25
depth 12      →  weight 0.125
depth 13      →  weight 0.0625
```

Hard cap of **13** (`config::MAX_TREE_DEPTH`), down from an earlier 18.

Rationale: every added depth level roughly doubles the upper bound on output frequency (each `sin(mul(…))` composition can multiply frequency), so deeper trees are disproportionately bad for "smooth art." The halving schedule means:

- ~91% of new genomes land at depth ≤ 9 (calm regime).
- ~5% at depth 10, ~2.5% at 11, ~1.3% at 12, ~0.6% at 13.
- Still some density at the complex end, so interesting deep expressions aren't impossible.

**Sampled per-genome, not per-individual** — each of the six trees in an Individual draws an independent depth. Good for diversity: one channel may be simple and another intricate.

The UI slider (`Settings → MaxDepth`) clamps the distribution from above — set it to 5 for a run of shallow trees, no recompile needed.

---

### 3. ScaledX / ScaledY range

`ScaledX` and `ScaledY` are terminal ops that multiply the coordinate by a random per-node factor drawn as:

```
value = 10 ^ rand(-0.5, 0.5)     →  factor ∈ [0.32, 3.16]
```

(Down from an earlier `10 ^ rand(-1, 1)` range of `[0.1, 10]`.)

A single `ScaledX` gives a coord swing of roughly `[-3, +3]` — a sine of that runs ~1 full cycle across the tile. Two nested `ScaledX`s in a `Mul` could still push frequency up by ~10×, but that's a fraction of what the old range permitted.

Mutated in both initial construction (`Node::random_terminal`, `Node::random_bounded`) and in expression-mutation's op-swap path.

---

### 4. FBM / noise octave count

All multi-octave noise ops (`FBM`, `Turbulence`, `Ridged`, `Billow`, `DomainWarp`) take an octave count in their `c_literal` field. Range:

```
c_literal ∈ 1..=4     (was 1..=8 / 1..=6)
```

Each octave doubles the base frequency, so 4 octaves = up to 16× base — 8 octaves would have been 256×. Four is enough to get recognizable fractal character without crossing into pure noise.

Set in three places: random node construction, subtree mutation on FBM (`rng.gen_range(1..=4)`), and expression-mutation when swapping into a noise op.

---

### 5. Color space as an evolvable trait

The final 3-channel → RGB conversion is chosen per-individual from 5 color models (HSV, RGB, HSL, CMY, YUV), inherited on breed, and rarely mutated (`COLOR_MODEL_MUTATION_PROB = 0.03`). This isn't strictly a frequency control, but it matters: different color models interpret the same three channels very differently, and letting selection pressure choose the model is cheap and expressive. See `config::NUM_COLOR_MODELS` and the `channels_to_rgb` switch in both `assets/shaders/compute.wgsl` and `src/app.rs`.

---

### 6. Soft viability filters

Before returning a new Individual, `random_with_depth` retries (up to 10 times) until the candidate passes two cheap checks:

- **Per-channel range/mean** (`channel_ok`): each of the three final channels must span at least `MIN_CHANNEL_RANGE = 0.05` and have mean ≥ `MIN_CHANNEL_MEAN = 0.05` across a 5×5 sample grid. Rejects channels that collapse to a constant.
- **Whole-tile viability** (`render_viable`): a 64×36 CPU render must have mean brightness ≥ `MIN_CHANNEL_MEAN`, stdev ≥ 0.02, and no more than 70% of pixels below 0.02 brightness. Rejects all-black or one-color renders.

These aren't frequency biases — they're hard filters against degenerate output. Shown here because they're part of the same "finger on the scale" philosophy: we'd rather re-roll than show the user a black tile.

---

## What we deliberately did **not** do

- **No tree-size penalty during selection.** Smaller trees would dominate in a few generations, collapsing diversity. The depth distribution already biases size indirectly and is gentler.
- **No narrowing of `Const`'s [0, 1) range.** Constants pass through many layers before affecting output; narrowing them has unpredictable downstream effects and is harder to reason about than narrowing scale factors directly.
- **No crossover bias.** Crossover currently just picks one parent's tree unchanged (see `evolution::crossover`). A future improvement would be real subtree exchange, but that interacts with the viability filters in ways we haven't tested yet.
- **No per-depth op weights.** It would be principled to lower `Mul` and `Fract` weights specifically at deep nodes, but a single flat weight is simpler and has been enough in practice.

---

## How to tune

If the population still looks too busy:

1. **Cap depth harder** — drop the Settings slider to 7–9. Usually the fastest fix.
2. **Lower `ScaledX`/`ScaledY` range further** — edit the `powf` ranges in `src/genome/node.rs` and `src/evolution.rs` to `-0.3..=0.3`.
3. **Drop noise weights to 0.3** — in `src/genome/op.rs`.
4. **Cap FBM octaves at 3** — in the three `c_literal = rng.gen_range(1..=4)` sites.

If the population looks too bland:

1. **Raise noise weights back to 1.0**.
2. **Raise depth cap** and shift the halving to start at 11 instead of 10.
3. **Boost `Mul` and `Fract`** — these create texture.

Each of these is a single-number change. Make one at a time; the effects compound fast and it's easy to over-correct.
