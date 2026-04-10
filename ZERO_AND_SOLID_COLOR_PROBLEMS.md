# Zero and Solid Color Problems

Analysis of why many generated images appear as solid colors.

---

## Root Causes Identified

### 1. `safe01()` Function Collapsing Values to Zero (`main.rs:51-53`)

```rust
fn safe01(v: f32) -> f32 {
    if v.is_finite() { v.rem_euclid(1.0) } else { 0.0 }
}
```

**The problem:** When evaluation produces `NaN`, `infinity`, or very large numbers that overflow, the function returns `0.0`. This is extremely common with:

- `Exp()` producing huge values → `rem_euclid` on infinity = `NaN` → returns `0.0`
- `Pow()` with negative bases → returns `0.0` (see `node.rs:159-163`)
- `Log()` of negative numbers → `NaN` → returns `0.0`

**Effect:** If any channel (H, S, or V) consistently evaluates to non-finite values across the entire image, you get:
- **S=0** → grayscale regardless of H and V
- **V=0** → pure black
- All channels = 0 → solid gray/black

---

### 2. `Node::random_bounded()` Has a Bug with Depth Tracking (`node.rs:57-129`)

The `current_depth` parameter is passed but **never actually used to limit recursion**:

```rust
fn random_bounded(rng: &mut impl Rng, max_depth: usize, max_size: usize) -> Self {
    let current_depth = 0;  // ← Always reset to 0!
    let remaining_budget = max_size;

    if remaining_budget < MIN_TREE_SIZE || current_depth >= max_depth {
        // This check never triggers based on depth because current_depth is always 0
```

**Consequences:**
- Trees grow based only on `remaining_budget` (size), not actual depth
- The budget calculation (`remaining_budget - 2` or `-3`) doesn't accurately track tree complexity
- Many trees become **shallow and degenerate**, collapsing to simple patterns like `Const(0.5)` or just `X`

---

### 3. HSV Conversion: S=0 Produces Grayscale (`main.rs:56-59`)

```rust
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    if s == 0.0 {
        let c = (v * 255.0) as u8;
        return [c, c, c];  // ← Solid grayscale!
    }
```

**The problem:** If the S channel evaluates to exactly `0.0` (or very close), the image becomes a **solid grayscale color**. This happens when:
- S channel tree is just `Const(0)` or similar
- S channel produces values that all round to 0 after `safe01()`

---

## Summary Table

| Cause | Effect | Frequency |
|-------|--------|-----------|
| `Exp()`/`Log()` overflow → `safe01()` returns 0 | Entire channel = 0 | High |
| S channel = 0.0 | Grayscale solid color | Medium |
| V channel = 0.0 | Pure black | Medium |
| Shallow/degenerate trees | Uniform or gradient-only patterns | High |
| `Pow()` with negative base returns 0 | Localized dead zones | Medium |

---

## Potential Solutions

### Immediate Fix: Clamp Constants to [0, 1]

**Rationale:** HSV only cares about values in the range [0, 1]. Limiting all constants to this range would:
- Make `Exp()` behave nicely: `exp(0) = 1`, `exp(1) ≈ 2.718` (manageable)
- Make `Log()` safe: `ln(0)` is still -∞, but `ln(x)` for x ∈ (0, 1] gives [-∞, 0]
- Prevent runaway growth from constant multiplication

**Tradeoff:** This restricts the expressiveness of evolved expressions. Some interesting patterns may require constants outside [0, 1].

### Alternative: Clamp After Evaluation

Instead of limiting constants, clamp intermediate results during `eval()`:
```rust
fn eval(&self, x: f32, y: f32) -> f32 {
    let result = match self {
        // ... existing logic ...
    };
    // Clamp to a reasonable range before returning
    result.clamp(-10.0, 10.0)
}
```

This prevents overflow while preserving constant expressiveness.

### Alternative: Improve `safe01()`

Instead of returning `0.0` for non-finite values, return a deterministic fallback based on position:
```rust
fn safe01(v: f32) -> f32 {
    if v.is_finite() { 
        v.rem_euclid(1.0) 
    } else { 
        // Return something that varies by pixel, not a constant
        0.5  // or use x/y coordinates for variation
    }
}
```

### Fix Depth Tracking Bug

Fix `node.rs:58` to actually track depth:
```rust
fn random_bounded(rng: &mut impl Rng, current_depth: usize, max_depth: usize, max_size: usize) -> Self {
    // Use current_depth parameter instead of hardcoding 0
}
```

This would produce more varied tree structures.
