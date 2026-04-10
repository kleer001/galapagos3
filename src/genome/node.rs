use rand::Rng;

/// Typed expression tree nodes per ROADMAP.md function set
#[derive(Clone, Debug)]
pub enum Node {
    /// Input: x coordinate (normalized -1 to 1)
    X,
    /// Input: y coordinate (normalized -1 to 1)
    Y,
    /// Constant value
    Const(f32),
    /// Sin(child)
    Sin(Box<Node>),
    /// Cos(child)
    Cos(Box<Node>),
    /// Tan(child)
    Tan(Box<Node>),
    /// Abs(child)
    Abs(Box<Node>),
    /// Sqrt(child)
    Sqrt(Box<Node>),
    /// Log(child)
    Log(Box<Node>),
    /// Exp(child)
    Exp(Box<Node>),
    /// Fract(child) - fractional part
    Fract(Box<Node>),
    /// Add(left, right)
    Add(Box<Node>, Box<Node>),
    /// Sub(left, right)
    Sub(Box<Node>, Box<Node>),
    /// Mul(left, right)
    Mul(Box<Node>, Box<Node>),
    /// Div(left, right)
    Div(Box<Node>, Box<Node>),
    /// Pow(base, exp)
    Pow(Box<Node>, Box<Node>),
    /// Mix(low, high, t) - lerp
    Mix(Box<Node>, Box<Node>, Box<Node>),
    /// Smoothstep(edge0, edge1, x)
    Smoothstep(Box<Node>, Box<Node>, Box<Node>),
    /// Length(v) - vector magnitude
    Length(Box<Node>),
    /// Dot(a, b) - dot product
    Dot(Box<Node>, Box<Node>),
    // Phase 2: Additional operators
    /// Acos(child) - inverse cosine
    Acos(Box<Node>),
    /// Asin(child) - inverse sine
    Asin(Box<Node>),
    /// Atan(child) - inverse tangent
    Atan(Box<Node>),
    /// Sinh(child) - hyperbolic sine
    Sinh(Box<Node>),
    /// Cosh(child) - hyperbolic cosine
    Cosh(Box<Node>),
    /// Tanh(child) - hyperbolic tangent
    Tanh(Box<Node>),
    /// Min(left, right) - minimum
    Min(Box<Node>, Box<Node>),
    /// Max(left, right) - maximum
    Max(Box<Node>, Box<Node>),
    /// Clamp(value, min, max) - clamp to range
    Clamp(Box<Node>, Box<Node>, Box<Node>),
    /// Sign(child) - sign function (-1, 0, or 1)
    Sign(Box<Node>),
    /// Floor(child) - floor function
    Floor(Box<Node>),
    /// Ceil(child) - ceiling function
    Ceil(Box<Node>),
    /// Round(child) - round to nearest integer
    Round(Box<Node>),
    /// Negate(child) - unary minus
    Negate(Box<Node>),
    /// Step(edge, x) - hard step function
    Step(Box<Node>, Box<Node>),
    /// Reciprocal(child) - 1/x
    Reciprocal(Box<Node>),
    /// Invert(child) - 1.0 - x
    Invert(Box<Node>),
    /// Radial() - distance from center (0,0)
    Radial,
}

const MAX_TREE_DEPTH: usize = 6;
const MIN_TREE_SIZE: usize = 3;
const MAX_TREE_SIZE: usize = 15;

impl Node {
    pub fn random(rng: &mut impl Rng) -> Self {
        Self::random_bounded(rng, MAX_TREE_DEPTH, MAX_TREE_SIZE)
    }

    fn random_bounded(rng: &mut impl Rng, max_depth: usize, max_size: usize) -> Self {
        let current_depth = 0;
        let remaining_budget = max_size;

        if remaining_budget < MIN_TREE_SIZE || current_depth >= max_depth {
            // Terminal node: always X or Y (guarantees coordinate dependency)
            match rng.gen_range(0..2) {
                0 => Node::X,
                _ => Node::Y,
            }
        } else {
            // 1% terminal rate: weighted random selection favoring operators over terminals
            if rng.gen_bool(0.01) {
                // Terminal: always X or Y
                match rng.gen_range(0..2) {
                    0 => Node::X,
                    _ => Node::Y,
                }
            } else {
                // Non-terminal operators (unary, binary, ternary)
                let op = rng.gen_range(3..45);

                match op {
                    // Phase 1 Unary operators (Sin, Cos, Tan, Abs, Sqrt, Log, Exp, Fract, Length)
                    3..=11 => {
                        let child = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 2));
                        match op {
                            3 => Node::Sin(child),
                            4 => Node::Cos(child),
                            5 => Node::Tan(child),
                            6 => Node::Abs(child),
                            7 => Node::Sqrt(child),
                            8 => Node::Log(child),
                            9 => Node::Exp(child),
                            10 => Node::Fract(child),
                            _ => Node::Length(child),
                        }
                    }
                    // Phase 2 Unary operators (Acos, Asin, Atan, Sinh, Cosh, Tanh, Sign, Floor, Ceil, Round, Negate, Reciprocal, Invert)
                    12..=24 => {
                        let child = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 2));
                        match op {
                            12 => Node::Acos(child),
                            13 => Node::Asin(child),
                            14 => Node::Atan(child),
                            15 => Node::Sinh(child),
                            16 => Node::Cosh(child),
                            17 => Node::Tanh(child),
                            18 => Node::Sign(child),
                            19 => Node::Floor(child),
                            20 => Node::Ceil(child),
                            21 => Node::Round(child),
                            22 => Node::Negate(child),
                            23 => Node::Reciprocal(child),
                            _ => Node::Invert(child),
                        }
                    }
                    // Phase 1 Binary operators (Add, Sub, Mul, Div, Pow, Mix, Dot)
                    25..=32 => {
                        let left = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 2));
                        let right = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 2));
                        match op {
                            25 => Node::Add(left, right),
                            26 => Node::Sub(left, right),
                            27 => Node::Mul(left, right),
                            28 => Node::Div(left, right),
                            29 => Node::Pow(left, right),
                            30 => Node::Mix(left, right, Box::new(Node::Const(rng.gen_range(0.0..1.0)))),
                            31 => Node::Dot(left, right),
                            _ => Node::Step(left, right), // Step as binary
                        }
                    }
                    // Phase 2 Binary operators (Min, Max, Clamp treated as binary with const)
                    33..=34 => {
                        let left = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 2));
                        let right = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 2));
                        match op {
                            33 => Node::Min(left, right),
                            _ => Node::Max(left, right),
                        }
                    }
                    // Ternary operators (Smoothstep, Clamp)
                    35..=36 => {
                        let a = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 3));
                        let b = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 3));
                        let c = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 3));
                        match op {
                            35 => Node::Smoothstep(a, b, c),
                            _ => Node::Clamp(a, b, c),
                        }
                    }
                    // Radial (no arguments)
                    37..=44 => Node::Radial,
                    _ => Node::Radial, // Fallback for any out-of-range values
                }
            }
        }
    }

    /// Evaluate the tree at (x, y)
    pub fn eval(&self, x: f32, y: f32) -> f32 {
        match self {
            Node::X => x,
            Node::Y => y,
            Node::Const(v) => *v,
            Node::Sin(child) => child.eval(x, y).sin(),
            Node::Cos(child) => child.eval(x, y).cos(),
            Node::Tan(child) => child.eval(x, y).tan(),
            Node::Abs(child) => child.eval(x, y).abs(),
            Node::Sqrt(child) => child.eval(x, y).sqrt().max(0.0),
            Node::Log(child) => child.eval(x, y).ln(),
            Node::Exp(child) => child.eval(x, y).exp(),
            Node::Fract(child) => child.eval(x, y).fract(),
            Node::Add(left, right) => left.eval(x, y) + right.eval(x, y),
            Node::Sub(left, right) => left.eval(x, y) - right.eval(x, y),
            Node::Mul(left, right) => left.eval(x, y) * right.eval(x, y),
            Node::Div(left, right) => {
                let denom = right.eval(x, y);
                if denom.abs() > 1e-6 {
                    left.eval(x, y) / denom
                } else {
                    0.0
                }
            }
            Node::Pow(base, exp) => {
                let b = base.eval(x, y);
                let e = exp.eval(x, y);
                if b > 0.0 {
                    b.powf(e)
                } else {
                    0.0
                }
            }
            Node::Mix(low, high, t) => low.eval(x, y) + (high.eval(x, y) - low.eval(x, y)) * t.eval(x, y),
            Node::Smoothstep(edge0, edge1, input) => {
                let x_val = input.eval(x, y);
                let e0 = edge0.eval(x, y);
                let e1 = edge1.eval(x, y);
                let t = (x_val - e0) / (e1 - e0);
                let t = t.clamp(0.0, 1.0);
                t * (t - 2.0) * t * t + 1.0
            }
            Node::Length(child) => child.eval(x, y).abs(),
            Node::Dot(left, right) => {
                // Simplified: treat as scalar dot product
                left.eval(x, y) * right.eval(x, y)
            }
            // Phase 2 operators
            Node::Acos(child) => {
                let v = child.eval(x, y).clamp(-1.0, 1.0);
                v.acos()
            }
            Node::Asin(child) => {
                let v = child.eval(x, y).clamp(-1.0, 1.0);
                v.asin()
            }
            Node::Atan(child) => child.eval(x, y).atan(),
            Node::Sinh(child) => child.eval(x, y).sinh(),
            Node::Cosh(child) => child.eval(x, y).cosh(),
            Node::Tanh(child) => child.eval(x, y).tanh(),
            Node::Min(left, right) => left.eval(x, y).min(right.eval(x, y)),
            Node::Max(left, right) => left.eval(x, y).max(right.eval(x, y)),
            Node::Clamp(value, min, max) => {
                let v = value.eval(x, y);
                let lo = min.eval(x, y);
                let hi = max.eval(x, y);
                v.clamp(lo, hi)
            }
            Node::Sign(child) => child.eval(x, y).copysign(1.0),
            Node::Floor(child) => child.eval(x, y).floor(),
            Node::Ceil(child) => child.eval(x, y).ceil(),
            Node::Round(child) => child.eval(x, y).round(),
            Node::Negate(child) => -child.eval(x, y),
            Node::Step(edge, x_node) => {
                let e = edge.eval(x, y);
                let xv = x_node.eval(x, y);
                if xv >= e { 1.0 } else { 0.0 }
            }
            Node::Reciprocal(child) => {
                let v = child.eval(x, y);
                if v.abs() > 1e-6 { 1.0 / v } else { 0.0 }
            }
            Node::Invert(child) => 1.0 - child.eval(x, y),
            Node::Radial => (x * x + y * y).sqrt(),
        }
    }
}
