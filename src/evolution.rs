use crate::config;
use crate::genome::{Genome, Instruction, OpCode, Node};
use rand::Rng;

// ============================================================================
// EVOLUTION PARAMETERS
// ============================================================================

pub const DEFAULT_SUBTREE_MUTATION_PROB: f64 = config::SUBTREE_MUTATION_PROB;
pub const DEFAULT_SUBTREE_STOP_PROB: f64 = config::SUBTREE_STOP_PROB;
pub const DEFAULT_BINARY_CHILD_SIDE_PROB: f64 = config::BINARY_CHILD_SIDE_PROB;
pub const DEFAULT_FRESH_RANDOM_COUNT: usize = config::FRESH_RANDOM_COUNT;
pub const DEFAULT_MAX_TREE_DEPTH: usize = config::MAX_TREE_DEPTH;

/// Runtime evolution parameters (can be modified during execution)
#[derive(Clone, Copy)]
pub struct EvolutionParams {
    pub subtree_mutation_prob: f64,
    pub subtree_stop_prob: f64,
    pub binary_child_side_prob: f64,
}

impl Default for EvolutionParams {
    fn default() -> Self {
        Self {
            subtree_mutation_prob: DEFAULT_SUBTREE_MUTATION_PROB,
            subtree_stop_prob: DEFAULT_SUBTREE_STOP_PROB,
            binary_child_side_prob: DEFAULT_BINARY_CHILD_SIDE_PROB,
        }
    }
}

pub fn mutate(genome: &Genome, rng: &mut impl Rng) -> Genome {
    mutate_with_params(genome, rng, &EvolutionParams::default())
}

pub fn mutate_with_params(genome: &Genome, rng: &mut impl Rng, params: &EvolutionParams) -> Genome {
    let mut tree = genome.tree();
    if rng.gen_bool(params.subtree_mutation_prob) {
        tree = mutate_subtree_with_params(&tree, rng, params);
    } else {
        tree = replace_node(&tree, rng);
    }
    Genome::new(tree)
}

// ============================================================================
// Mutation helpers — reduce boilerplate in mutate_subtree_with_params
// ============================================================================

fn mutate_unary<F: Fn(Box<Node>) -> Node>(
    c: &Box<Node>, rng: &mut impl Rng, params: &EvolutionParams, ctor: F,
) -> Node {
    ctor(Box::new(mutate_subtree_with_params(c.as_ref(), rng, params)))
}

/// Recurse into left or right child based on binary_child_side_prob.
fn mutate_binary<F: Fn(Box<Node>, Box<Node>) -> Node>(
    l: &Box<Node>, r: &Box<Node>, rng: &mut impl Rng, params: &EvolutionParams, ctor: F,
) -> Node {
    if rng.gen_bool(params.binary_child_side_prob) {
        ctor(Box::new(mutate_subtree_with_params(l.as_ref(), rng, params)), r.clone())
    } else {
        ctor(l.clone(), Box::new(mutate_subtree_with_params(r.as_ref(), rng, params)))
    }
}

/// Recurse into one of three children, chosen uniformly.
fn mutate_ternary<F: Fn(Box<Node>, Box<Node>, Box<Node>) -> Node>(
    a: &Box<Node>, b: &Box<Node>, c: &Box<Node>, rng: &mut impl Rng, params: &EvolutionParams, ctor: F,
) -> Node {
    match rng.gen_range(0..3) {
        0 => ctor(Box::new(mutate_subtree_with_params(a.as_ref(), rng, params)), b.clone(), c.clone()),
        1 => ctor(a.clone(), Box::new(mutate_subtree_with_params(b.as_ref(), rng, params)), c.clone()),
        _ => ctor(a.clone(), b.clone(), Box::new(mutate_subtree_with_params(c.as_ref(), rng, params))),
    }
}

fn mutate_subtree_with_params(node: &Node, rng: &mut impl Rng, params: &EvolutionParams) -> Node {
    // Stop recursion with given probability — keeps ~1-2 nodes changed per call
    if rng.gen_bool(params.subtree_stop_prob) {
        return node.clone();
    }
    match node {
        Node::X | Node::Y => node.clone(),
        Node::Const(_) => Node::Const(rng.gen::<f32>()),
        // Unary operators
        Node::Sin(c)        => mutate_unary(c, rng, params, Node::Sin),
        Node::Cos(c)        => mutate_unary(c, rng, params, Node::Cos),
        Node::Tan(c)        => mutate_unary(c, rng, params, Node::Tan),
        Node::Abs(c)        => mutate_unary(c, rng, params, Node::Abs),
        Node::Sqrt(c)       => mutate_unary(c, rng, params, Node::Sqrt),
        Node::Log(c)        => mutate_unary(c, rng, params, Node::Log),
        Node::Exp(c)        => mutate_unary(c, rng, params, Node::Exp),
        Node::Fract(c)      => mutate_unary(c, rng, params, Node::Fract),
        Node::Length(c)     => mutate_unary(c, rng, params, Node::Length),
        Node::Acos(c)       => mutate_unary(c, rng, params, Node::Acos),
        Node::Asin(c)       => mutate_unary(c, rng, params, Node::Asin),
        Node::Atan(c)       => mutate_unary(c, rng, params, Node::Atan),
        Node::Sinh(c)       => mutate_unary(c, rng, params, Node::Sinh),
        Node::Cosh(c)       => mutate_unary(c, rng, params, Node::Cosh),
        Node::Tanh(c)       => mutate_unary(c, rng, params, Node::Tanh),
        Node::Sign(c)       => mutate_unary(c, rng, params, Node::Sign),
        Node::Floor(c)      => mutate_unary(c, rng, params, Node::Floor),
        Node::Ceil(c)       => mutate_unary(c, rng, params, Node::Ceil),
        Node::Round(c)      => mutate_unary(c, rng, params, Node::Round),
        Node::Negate(c)     => mutate_unary(c, rng, params, Node::Negate),
        Node::Reciprocal(c) => mutate_unary(c, rng, params, Node::Reciprocal),
        Node::Invert(c)     => mutate_unary(c, rng, params, Node::Invert),
        // Binary operators
        Node::Add(l, r)        => mutate_binary(l, r, rng, params, Node::Add),
        Node::Sub(l, r)        => mutate_binary(l, r, rng, params, Node::Sub),
        Node::Mul(l, r)        => mutate_binary(l, r, rng, params, Node::Mul),
        Node::Div(l, r)        => mutate_binary(l, r, rng, params, Node::Div),
        Node::Pow(l, r)        => mutate_binary(l, r, rng, params, Node::Pow),
        Node::Dot(l, r)        => mutate_binary(l, r, rng, params, Node::Dot),
        Node::Min(l, r)        => mutate_binary(l, r, rng, params, Node::Min),
        Node::Max(l, r)        => mutate_binary(l, r, rng, params, Node::Max),
        Node::Step(l, r)       => mutate_binary(l, r, rng, params, Node::Step),
        Node::ValueNoise(l, r) => mutate_binary(l, r, rng, params, Node::ValueNoise),
        Node::WarpX(l, r)      => mutate_binary(l, r, rng, params, Node::WarpX),
        Node::WarpY(l, r)      => mutate_binary(l, r, rng, params, Node::WarpY),
        // Ternary operators
        Node::Smoothstep(e0, e1, x) => mutate_ternary(e0, e1, x, rng, params, Node::Smoothstep),
        Node::Clamp(v, lo, hi)      => mutate_ternary(v, lo, hi, rng, params, Node::Clamp),
        // Mix: intentionally only mutates low/high, never t (preserves blend control)
        Node::Mix(low, high, t) => mutate_binary(low, high, rng, params, |l, r| Node::Mix(l, r, t.clone())),
        // FBM: third field is octave count (i32), not a child node
        Node::FBM(x, y, octaves) => match rng.gen_range(0..3) {
            0 => Node::FBM(Box::new(mutate_subtree_with_params(x.as_ref(), rng, params)), y.clone(), *octaves),
            1 => Node::FBM(x.clone(), Box::new(mutate_subtree_with_params(y.as_ref(), rng, params)), *octaves),
            _ => Node::FBM(x.clone(), y.clone(), rng.gen_range(1..=8)),
        },
        Node::MirrorX => Node::MirrorX,
        Node::MirrorY => Node::MirrorY,
    }
}

// ============================================================================
// Replace helpers — reduce boilerplate in replace_node
// ============================================================================

fn replace_unary<F: Fn(Box<Node>) -> Node>(c: &Box<Node>, rng: &mut impl Rng, ctor: F) -> Node {
    ctor(Box::new(replace_node(c.as_ref(), rng)))
}

/// Recurse into left, right, or replace whole node (1/3 chance each).
fn replace_binary<F: Fn(Box<Node>, Box<Node>) -> Node>(
    l: &Box<Node>, r: &Box<Node>, rng: &mut impl Rng, ctor: F,
) -> Node {
    match rng.gen_range(0..3) {
        0 => ctor(Box::new(replace_node(l.as_ref(), rng)), r.clone()),
        1 => ctor(l.clone(), Box::new(replace_node(r.as_ref(), rng))),
        _ => Node::random(rng),
    }
}

/// Recurse into one of three children, or replace whole node (1/4 chance).
fn replace_ternary<F: Fn(Box<Node>, Box<Node>, Box<Node>) -> Node>(
    a: &Box<Node>, b: &Box<Node>, c: &Box<Node>, rng: &mut impl Rng, ctor: F,
) -> Node {
    match rng.gen_range(0..4) {
        0 => ctor(Box::new(replace_node(a.as_ref(), rng)), b.clone(), c.clone()),
        1 => ctor(a.clone(), Box::new(replace_node(b.as_ref(), rng)), c.clone()),
        2 => ctor(a.clone(), b.clone(), Box::new(replace_node(c.as_ref(), rng))),
        _ => Node::random(rng),
    }
}

fn replace_node(node: &Node, rng: &mut impl Rng) -> Node {
    match node {
        Node::X | Node::Y => node.clone(),
        Node::Const(_) => Node::random(rng),
        // Unary operators
        Node::Sin(c)        => replace_unary(c, rng, Node::Sin),
        Node::Cos(c)        => replace_unary(c, rng, Node::Cos),
        Node::Tan(c)        => replace_unary(c, rng, Node::Tan),
        Node::Abs(c)        => replace_unary(c, rng, Node::Abs),
        Node::Sqrt(c)       => replace_unary(c, rng, Node::Sqrt),
        Node::Log(c)        => replace_unary(c, rng, Node::Log),
        Node::Exp(c)        => replace_unary(c, rng, Node::Exp),
        Node::Fract(c)      => replace_unary(c, rng, Node::Fract),
        Node::Length(c)     => replace_unary(c, rng, Node::Length),
        Node::Acos(c)       => replace_unary(c, rng, Node::Acos),
        Node::Asin(c)       => replace_unary(c, rng, Node::Asin),
        Node::Atan(c)       => replace_unary(c, rng, Node::Atan),
        Node::Sinh(c)       => replace_unary(c, rng, Node::Sinh),
        Node::Cosh(c)       => replace_unary(c, rng, Node::Cosh),
        Node::Tanh(c)       => replace_unary(c, rng, Node::Tanh),
        Node::Sign(c)       => replace_unary(c, rng, Node::Sign),
        Node::Floor(c)      => replace_unary(c, rng, Node::Floor),
        Node::Ceil(c)       => replace_unary(c, rng, Node::Ceil),
        Node::Round(c)      => replace_unary(c, rng, Node::Round),
        Node::Negate(c)     => replace_unary(c, rng, Node::Negate),
        Node::Reciprocal(c) => replace_unary(c, rng, Node::Reciprocal),
        Node::Invert(c)     => replace_unary(c, rng, Node::Invert),
        // Binary operators
        Node::Add(l, r)        => replace_binary(l, r, rng, Node::Add),
        Node::Sub(l, r)        => replace_binary(l, r, rng, Node::Sub),
        Node::Mul(l, r)        => replace_binary(l, r, rng, Node::Mul),
        Node::Div(l, r)        => replace_binary(l, r, rng, Node::Div),
        Node::Pow(l, r)        => replace_binary(l, r, rng, Node::Pow),
        Node::Dot(l, r)        => replace_binary(l, r, rng, Node::Dot),
        Node::Min(l, r)        => replace_binary(l, r, rng, Node::Min),
        Node::Max(l, r)        => replace_binary(l, r, rng, Node::Max),
        Node::Step(l, r)       => replace_binary(l, r, rng, Node::Step),
        Node::ValueNoise(l, r) => replace_binary(l, r, rng, Node::ValueNoise),
        Node::WarpX(l, r)      => replace_binary(l, r, rng, Node::WarpX),
        Node::WarpY(l, r)      => replace_binary(l, r, rng, Node::WarpY),
        // Ternary operators
        Node::Smoothstep(e0, e1, x) => replace_ternary(e0, e1, x, rng, Node::Smoothstep),
        Node::Clamp(v, lo, hi)      => replace_ternary(v, lo, hi, rng, Node::Clamp),
        // Mix: intentionally only replaces low/high, never t
        Node::Mix(low, high, t) => match rng.gen_range(0..3) {
            0 => Node::Mix(Box::new(replace_node(low.as_ref(), rng)), high.clone(), t.clone()),
            1 => Node::Mix(low.clone(), Box::new(replace_node(high.as_ref(), rng)), t.clone()),
            _ => Node::random(rng),
        },
        // FBM: third field is octave count, not a child node
        Node::FBM(x, y, octaves) => match rng.gen_range(0..3) {
            0 => Node::FBM(Box::new(replace_node(x.as_ref(), rng)), y.clone(), *octaves),
            1 => Node::FBM(x.clone(), Box::new(replace_node(y.as_ref(), rng)), *octaves),
            _ => Node::random(rng),
        },
        Node::MirrorX => Node::random(rng),
        Node::MirrorY => Node::random(rng),
    }
}

pub fn crossover(a: &Genome, b: &Genome, rng: &mut impl Rng) -> Genome {
    // NOTE: conservative crossover — result is one full parent, no subtree mixing.
    let tree = if rng.gen_bool(0.5) { a.tree() } else { b.tree() };
    Genome::new(tree)
}

pub fn selection(population: &[Genome], rng: &mut impl Rng) -> Genome {
    let size = population.len();
    if size == 1 {
        return population[0].clone();
    }
    population[rng.gen_range(0..size)].clone()
}

impl Genome {
    pub fn tree(&self) -> Node {
        instructions_to_tree(&self.instructions)
    }
}

// ============================================================================
// Stack push helpers — reduce boilerplate in instructions_to_tree
// ============================================================================

fn push_unary<F: Fn(Box<Node>) -> Node>(stack: &mut Vec<Option<Node>>, idx: usize, ctor: F) {
    if idx < stack.len() {
        if let Some(child) = stack[idx].clone() {
            stack.push(Some(ctor(Box::new(child))));
        }
    }
}

fn push_binary<F: Fn(Box<Node>, Box<Node>) -> Node>(
    stack: &mut Vec<Option<Node>>, a: usize, b: usize, ctor: F,
) {
    if a < stack.len() && b < stack.len() {
        if let (Some(x), Some(y)) = (stack[a].clone(), stack[b].clone()) {
            stack.push(Some(ctor(Box::new(x), Box::new(y))));
        }
    }
}

fn push_ternary<F: Fn(Box<Node>, Box<Node>, Box<Node>) -> Node>(
    stack: &mut Vec<Option<Node>>, a: usize, b: usize, c: usize, ctor: F,
) {
    if a < stack.len() && b < stack.len() && c < stack.len() {
        if let (Some(x), Some(y), Some(z)) = (stack[a].clone(), stack[b].clone(), stack[c].clone()) {
            stack.push(Some(ctor(Box::new(x), Box::new(y), Box::new(z))));
        }
    }
}

fn instructions_to_tree(instructions: &[Instruction]) -> Node {
    let mut stack: Vec<Option<Node>> = Vec::new();

    let real_end = instructions.iter().rposition(|i| i.op != OpCode::Const).unwrap_or(0);

    for instr in &instructions[..=real_end] {
        let (a, b, c) = (instr.a as usize, instr.b as usize, instr.c as usize);
        match instr.op {
            OpCode::X     => stack.push(Some(Node::X)),
            OpCode::Y     => stack.push(Some(Node::Y)),
            OpCode::Const => stack.push(Some(Node::Const(instr.value))),
            // Unary operators
            OpCode::Sin        => push_unary(&mut stack, a, Node::Sin),
            OpCode::Cos        => push_unary(&mut stack, a, Node::Cos),
            OpCode::Tan        => push_unary(&mut stack, a, Node::Tan),
            OpCode::Abs        => push_unary(&mut stack, a, Node::Abs),
            OpCode::Sqrt       => push_unary(&mut stack, a, Node::Sqrt),
            OpCode::Log        => push_unary(&mut stack, a, Node::Log),
            OpCode::Exp        => push_unary(&mut stack, a, Node::Exp),
            OpCode::Fract      => push_unary(&mut stack, a, Node::Fract),
            OpCode::Length     => push_unary(&mut stack, a, Node::Length),
            OpCode::Acos       => push_unary(&mut stack, a, Node::Acos),
            OpCode::Asin       => push_unary(&mut stack, a, Node::Asin),
            OpCode::Atan       => push_unary(&mut stack, a, Node::Atan),
            OpCode::Sinh       => push_unary(&mut stack, a, Node::Sinh),
            OpCode::Cosh       => push_unary(&mut stack, a, Node::Cosh),
            OpCode::Tanh       => push_unary(&mut stack, a, Node::Tanh),
            OpCode::Sign       => push_unary(&mut stack, a, Node::Sign),
            OpCode::Floor      => push_unary(&mut stack, a, Node::Floor),
            OpCode::Ceil       => push_unary(&mut stack, a, Node::Ceil),
            OpCode::Round      => push_unary(&mut stack, a, Node::Round),
            OpCode::Negate     => push_unary(&mut stack, a, Node::Negate),
            OpCode::Reciprocal => push_unary(&mut stack, a, Node::Reciprocal),
            OpCode::Invert     => push_unary(&mut stack, a, Node::Invert),
            // Binary operators
            OpCode::Add        => push_binary(&mut stack, a, b, Node::Add),
            OpCode::Sub        => push_binary(&mut stack, a, b, Node::Sub),
            OpCode::Mul        => push_binary(&mut stack, a, b, Node::Mul),
            OpCode::Div        => push_binary(&mut stack, a, b, Node::Div),
            OpCode::Pow        => push_binary(&mut stack, a, b, Node::Pow),
            OpCode::Dot        => push_binary(&mut stack, a, b, Node::Dot),
            OpCode::Min        => push_binary(&mut stack, a, b, Node::Min),
            OpCode::Max        => push_binary(&mut stack, a, b, Node::Max),
            OpCode::Step       => push_binary(&mut stack, a, b, Node::Step),
            OpCode::ValueNoise => push_binary(&mut stack, a, b, Node::ValueNoise),
            OpCode::WarpX      => push_binary(&mut stack, a, b, Node::WarpX),
            OpCode::WarpY      => push_binary(&mut stack, a, b, Node::WarpY),
            // Ternary operators
            OpCode::Mix        => push_ternary(&mut stack, a, b, c, Node::Mix),
            OpCode::Smoothstep => push_ternary(&mut stack, a, b, c, Node::Smoothstep),
            OpCode::Clamp      => push_ternary(&mut stack, a, b, c, Node::Clamp),
            // FBM: third field is octave count stored in instr.c, not a stack index
            OpCode::FBM => {
                if a < stack.len() && b < stack.len() {
                    if let (Some(x), Some(y)) = (stack[a].clone(), stack[b].clone()) {
                        stack.push(Some(Node::FBM(Box::new(x), Box::new(y), instr.c)));
                    }
                }
            }
            OpCode::MirrorX => stack.push(Some(Node::MirrorX)),
            OpCode::MirrorY => stack.push(Some(Node::MirrorY)),
        }
    }

    stack.last().and_then(|n| n.clone()).unwrap_or(Node::Const(0.0))
}
