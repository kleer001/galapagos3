use super::linear::OpCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arity {
    Nullary,
    Unary,
    Binary,
    Ternary,
}

impl Arity {
    pub fn child_count(self) -> usize {
        match self {
            Arity::Nullary => 0,
            Arity::Unary => 1,
            Arity::Binary => 2,
            Arity::Ternary => 3,
        }
    }
}

/// CPU eval dispatch — variant determines how stack values map to the function.
pub enum EvalFn {
    /// f(x, y, stored_value) for coordinate inputs and constants
    Nullary(fn(f32, f32, f32) -> f32),
    /// f(a) for single-input ops
    Unary(fn(f32) -> f32),
    /// f(a, b) for two-input ops
    Binary(fn(f32, f32) -> f32),
    /// f(a, b, c) for three-input ops
    Ternary(fn(f32, f32, f32) -> f32),
    /// f(a, b, literal) — binary inputs plus integer literal (FBM octaves)
    BinaryLiteral(fn(f32, f32, i32) -> f32),
}

pub struct OpDef {
    pub opcode: OpCode,
    pub name: &'static str,
    pub arity: Arity,
    pub eval: EvalFn,
}

// ============================================================================
// Eval functions — guarded or non-trivial ops get named functions
// ============================================================================

fn eval_x(x: f32, _y: f32, _v: f32) -> f32 { x }
fn eval_y(_x: f32, y: f32, _v: f32) -> f32 { y }
fn eval_const(_x: f32, _y: f32, v: f32) -> f32 { v }
fn eval_mirror_x(x: f32, _y: f32, _v: f32) -> f32 { x.abs() }
fn eval_mirror_y(_x: f32, y: f32, _v: f32) -> f32 { y.abs() }

fn eval_sqrt(v: f32) -> f32 { v.sqrt().max(0.0) }
fn eval_acos(v: f32) -> f32 { v.clamp(-1.0, 1.0).acos() }
fn eval_asin(v: f32) -> f32 { v.clamp(-1.0, 1.0).asin() }
fn eval_sign(v: f32) -> f32 { v.copysign(1.0) }
fn eval_negate(v: f32) -> f32 { -v }
fn eval_reciprocal(v: f32) -> f32 { if v.abs() >= 1e-6 { 1.0 / v } else { 0.0 } }
fn eval_invert(v: f32) -> f32 { 1.0 - v }

fn eval_add(a: f32, b: f32) -> f32 { a + b }
fn eval_sub(a: f32, b: f32) -> f32 { a - b }
fn eval_mul(a: f32, b: f32) -> f32 { a * b }
fn eval_div(a: f32, b: f32) -> f32 { if b.abs() >= 1e-6 { a / b } else { 0.0 } }
fn eval_pow(a: f32, b: f32) -> f32 { if a > 0.0 { a.powf(b) } else { 0.0 } }
fn eval_step(edge: f32, x: f32) -> f32 { if x >= edge { 1.0 } else { 0.0 } }

fn eval_mix(lo: f32, hi: f32, t: f32) -> f32 { lo + (hi - lo) * t }
fn eval_smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn eval_atan2(a: f32, b: f32) -> f32 { a.atan2(b) }
fn eval_mod(a: f32, b: f32) -> f32 { if b.abs() >= 1e-6 { a - b * (a / b).floor() } else { 0.0 } }
fn eval_triwave(v: f32) -> f32 { ((v * 0.5).fract() * 2.0 - 1.0).abs() }
fn eval_chebyshev(a: f32, b: f32) -> f32 { a.abs().max(b.abs()) }
fn eval_manhattan(a: f32, b: f32) -> f32 { a.abs() + b.abs() }
fn eval_sinfold(v: f32) -> f32 { (v * std::f32::consts::PI).sin() }

fn eval_worley(vx: f32, vy: f32) -> f32 {
    let xi = vx.floor();
    let yi = vy.floor();
    let mut min_dist = f32::MAX;
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            let cx = xi + dx as f32;
            let cy = yi + dy as f32;
            let h1 = ((cx * 127.1 + cy * 311.3).sin() + 1.0) * 0.5;
            let h2 = ((cx * 269.5 + cy * 183.3).sin() + 1.0) * 0.5;
            let px = cx + h1;
            let py = cy + h2;
            let d = ((vx - px) * (vx - px) + (vy - py) * (vy - py)).sqrt();
            if d < min_dist { min_dist = d; }
        }
    }
    min_dist.min(1.0)
}

fn eval_value_noise(vx: f32, vy: f32) -> f32 {
    let xi = vx.floor();
    let yi = vy.floor();
    let fx = vx - xi;
    let fy = vy - yi;
    let hash = |ix: f32, iy: f32| -> f32 {
        let h = (ix * 127.1 + iy * 311.3).sin().cos();
        (h + 1.0) * 0.5
    };
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let top = lerp(hash(xi, yi), hash(xi + 1.0, yi), fx);
    let bottom = lerp(hash(xi, yi + 1.0), hash(xi + 1.0, yi + 1.0), fx);
    lerp(top, bottom, fy)
}

fn eval_fbm(vx: f32, vy: f32, octaves: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_val = 0.0;
    for _ in 0..octaves.clamp(1, 8) {
        let sx = vx * frequency;
        let sy = vy * frequency;
        let xi = sx.floor();
        let yi = sy.floor();
        let fx = sx - xi;
        let fy = sy - yi;
        let hash = |ix: f32, iy: f32| -> f32 {
            let h = (ix * 127.1 + iy * 311.3).sin().cos();
            (h + 1.0) * 0.5
        };
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
        let top = lerp(hash(xi, yi), hash(xi + 1.0, yi), fx);
        let bottom = lerp(hash(xi, yi + 1.0), hash(xi + 1.0, yi + 1.0), fx);
        let noise = lerp(top, bottom, fy);
        value += noise * amplitude;
        max_val += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    if max_val > 0.0 { value / max_val } else { 0.0 }
}

// ============================================================================
// THE REGISTRY — single source of truth for all operations
//
// To add a new op: add one entry here + one OpCode variant + WGSL case.
// To disable an op: comment out or remove its line.
// ============================================================================

pub static OP_REGISTRY: [OpDef; 46] = [
    // Phase 1: Core
    OpDef { opcode: OpCode::X,          name: "X",          arity: Arity::Nullary,  eval: EvalFn::Nullary(eval_x) },
    OpDef { opcode: OpCode::Y,          name: "Y",          arity: Arity::Nullary,  eval: EvalFn::Nullary(eval_y) },
    OpDef { opcode: OpCode::Const,      name: "Const",      arity: Arity::Nullary,  eval: EvalFn::Nullary(eval_const) },
    OpDef { opcode: OpCode::Sin,        name: "Sin",        arity: Arity::Unary,    eval: EvalFn::Unary(f32::sin) },
    OpDef { opcode: OpCode::Cos,        name: "Cos",        arity: Arity::Unary,    eval: EvalFn::Unary(f32::cos) },
    OpDef { opcode: OpCode::Tan,        name: "Tan",        arity: Arity::Unary,    eval: EvalFn::Unary(f32::tan) },
    OpDef { opcode: OpCode::Abs,        name: "Abs",        arity: Arity::Unary,    eval: EvalFn::Unary(f32::abs) },
    OpDef { opcode: OpCode::Sqrt,       name: "Sqrt",       arity: Arity::Unary,    eval: EvalFn::Unary(eval_sqrt) },
    OpDef { opcode: OpCode::Log,        name: "Log",        arity: Arity::Unary,    eval: EvalFn::Unary(f32::ln) },
    OpDef { opcode: OpCode::Exp,        name: "Exp",        arity: Arity::Unary,    eval: EvalFn::Unary(f32::exp) },
    OpDef { opcode: OpCode::Fract,      name: "Fract",      arity: Arity::Unary,    eval: EvalFn::Unary(f32::fract) },
    OpDef { opcode: OpCode::Add,        name: "Add",        arity: Arity::Binary,   eval: EvalFn::Binary(eval_add) },
    OpDef { opcode: OpCode::Sub,        name: "Sub",        arity: Arity::Binary,   eval: EvalFn::Binary(eval_sub) },
    OpDef { opcode: OpCode::Mul,        name: "Mul",        arity: Arity::Binary,   eval: EvalFn::Binary(eval_mul) },
    OpDef { opcode: OpCode::Div,        name: "Div",        arity: Arity::Binary,   eval: EvalFn::Binary(eval_div) },
    OpDef { opcode: OpCode::Pow,        name: "Pow",        arity: Arity::Binary,   eval: EvalFn::Binary(eval_pow) },
    OpDef { opcode: OpCode::Mix,        name: "Mix",        arity: Arity::Ternary,  eval: EvalFn::Ternary(eval_mix) },
    OpDef { opcode: OpCode::Smoothstep, name: "Smoothstep", arity: Arity::Ternary,  eval: EvalFn::Ternary(eval_smoothstep) },
    // Phase 2: Extended math
    OpDef { opcode: OpCode::Acos,       name: "Acos",       arity: Arity::Unary,    eval: EvalFn::Unary(eval_acos) },
    OpDef { opcode: OpCode::Asin,       name: "Asin",       arity: Arity::Unary,    eval: EvalFn::Unary(eval_asin) },
    OpDef { opcode: OpCode::Atan,       name: "Atan",       arity: Arity::Unary,    eval: EvalFn::Unary(f32::atan) },
    OpDef { opcode: OpCode::Sinh,       name: "Sinh",       arity: Arity::Unary,    eval: EvalFn::Unary(f32::sinh) },
    OpDef { opcode: OpCode::Cosh,       name: "Cosh",       arity: Arity::Unary,    eval: EvalFn::Unary(f32::cosh) },
    OpDef { opcode: OpCode::Tanh,       name: "Tanh",       arity: Arity::Unary,    eval: EvalFn::Unary(f32::tanh) },
    OpDef { opcode: OpCode::Min,        name: "Min",        arity: Arity::Binary,   eval: EvalFn::Binary(f32::min) },
    OpDef { opcode: OpCode::Max,        name: "Max",        arity: Arity::Binary,   eval: EvalFn::Binary(f32::max) },
    OpDef { opcode: OpCode::Clamp,      name: "Clamp",      arity: Arity::Ternary,  eval: EvalFn::Ternary(f32::clamp) },
    OpDef { opcode: OpCode::Sign,       name: "Sign",       arity: Arity::Unary,    eval: EvalFn::Unary(eval_sign) },
    OpDef { opcode: OpCode::Floor,      name: "Floor",      arity: Arity::Unary,    eval: EvalFn::Unary(f32::floor) },
    OpDef { opcode: OpCode::Ceil,       name: "Ceil",       arity: Arity::Unary,    eval: EvalFn::Unary(f32::ceil) },
    OpDef { opcode: OpCode::Round,      name: "Round",      arity: Arity::Unary,    eval: EvalFn::Unary(f32::round) },
    OpDef { opcode: OpCode::Negate,     name: "Negate",     arity: Arity::Unary,    eval: EvalFn::Unary(eval_negate) },
    OpDef { opcode: OpCode::Step,       name: "Step",       arity: Arity::Binary,   eval: EvalFn::Binary(eval_step) },
    OpDef { opcode: OpCode::Reciprocal, name: "Reciprocal", arity: Arity::Unary,    eval: EvalFn::Unary(eval_reciprocal) },
    OpDef { opcode: OpCode::Invert,     name: "Invert",     arity: Arity::Unary,    eval: EvalFn::Unary(eval_invert) },
    // Phase 3: Noise & spatial
    OpDef { opcode: OpCode::ValueNoise, name: "ValueNoise", arity: Arity::Binary,   eval: EvalFn::Binary(eval_value_noise) },
    OpDef { opcode: OpCode::FBM,        name: "FBM",        arity: Arity::Binary,   eval: EvalFn::BinaryLiteral(eval_fbm) },
    OpDef { opcode: OpCode::MirrorX,    name: "MirrorX",    arity: Arity::Nullary,  eval: EvalFn::Nullary(eval_mirror_x) },
    OpDef { opcode: OpCode::MirrorY,    name: "MirrorY",    arity: Arity::Nullary,  eval: EvalFn::Nullary(eval_mirror_y) },
    // New operators
    OpDef { opcode: OpCode::Atan2,      name: "Atan2",      arity: Arity::Binary,   eval: EvalFn::Binary(eval_atan2) },
    OpDef { opcode: OpCode::Mod,        name: "Mod",        arity: Arity::Binary,   eval: EvalFn::Binary(eval_mod) },
    OpDef { opcode: OpCode::Worley,     name: "Worley",     arity: Arity::Binary,   eval: EvalFn::Binary(eval_worley) },
    OpDef { opcode: OpCode::TriWave,    name: "TriWave",    arity: Arity::Unary,    eval: EvalFn::Unary(eval_triwave) },
    OpDef { opcode: OpCode::Chebyshev,  name: "Chebyshev",  arity: Arity::Binary,   eval: EvalFn::Binary(eval_chebyshev) },
    OpDef { opcode: OpCode::Manhattan,  name: "Manhattan",  arity: Arity::Binary,   eval: EvalFn::Binary(eval_manhattan) },
    OpDef { opcode: OpCode::SinFold,    name: "SinFold",    arity: Arity::Unary,    eval: EvalFn::Unary(eval_sinfold) },
];

/// Look up an op's definition by opcode. Indexed by discriminant value.
pub fn op_def(opcode: OpCode) -> &'static OpDef {
    &OP_REGISTRY[opcode as usize]
}
