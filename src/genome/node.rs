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
}

const MAX_TREE_DEPTH: usize = 6;
const MIN_TREE_SIZE: usize = 3;
const MAX_TREE_SIZE: usize = 15;

impl Node {
    pub fn random(rng: &mut impl Rng) -> Self {
        let tree = Self::random_bounded(rng, MAX_TREE_DEPTH, MAX_TREE_SIZE);
        eprintln!("Generated tree: {:?}", tree);
        tree
    }

    fn random_bounded(rng: &mut impl Rng, max_depth: usize, max_size: usize) -> Self {
        let current_depth = 0;
        let remaining_budget = max_size;

        if remaining_budget < MIN_TREE_SIZE || current_depth >= max_depth {
            // Terminal node: X, Y, or Const
            let choice = rng.gen_range(0..3);
            match choice {
                0 => Node::X,
                1 => Node::Y,
                _ => Node::Const(rng.gen_range(-5.0..5.0)),
            }
        } else {
            // Choose an operator based on remaining budget
            let op = rng.gen_range(0..20);

            match op {
                // Terminal inputs: X, Y
                0..=1 => {
                    if current_depth < max_depth - 1 {
                        // Wrap X/Y in a function for more interesting patterns
                        let choice = rng.gen_range(0..2);
                        let child = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 2));
                        match choice {
                            0 => Node::Sin(child),
                            _ => Node::Cos(child),
                        }
                    } else {
                        if rng.gen_bool(0.5) { Node::X } else { Node::Y }
                    }
                }
                // Constant
                2 => Node::Const(rng.gen_range(-5.0..5.0)),
                // Unary operators (Sin, Cos, Tan, Abs, Sqrt, Log, Exp, Fract, Length)
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
                // Binary operators (Add, Sub, Mul, Div, Pow, Mix)
                12..=17 => {
                    let left = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 2));
                    let right = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 2));
                    match op {
                        12 => Node::Add(left, right),
                        13 => Node::Sub(left, right),
                        14 => Node::Mul(left, right),
                        15 => Node::Div(left, right),
                        16 => Node::Pow(left, right),
                        _ => Node::Mix(left, right, Box::new(Node::Const(rng.gen_range(0.0..1.0)))),
                    }
                }
                // Ternary operator (Smoothstep)
                18 => {
                    let left = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 3));
                    Node::Smoothstep(
                        Box::new(Node::Const(-1.0)),
                        Box::new(Node::Const(1.0)),
                        left,
                    )
                }
                // Dot product (simplified scalar)
                _ => {
                    let left = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 2));
                    let right = Box::new(Self::random_bounded(rng, current_depth + 1, remaining_budget - 2));
                    Node::Dot(left, right)
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
        }
    }
}
