use crate::genome::{Genome, Instruction, OpCode, Node};
use rand::Rng;

// ============================================================================
// EVOLUTION PARAMETERS
// ============================================================================

/// 90% fine-tuning (mutate_subtree preserves structure), 10% disruptive (replace_node).
pub const SUBTREE_MUTATION_PROB: f64 = 0.999;

/// At each interior node in mutate_subtree, probability of stopping recursion early.
/// Limits how many nodes change per mutation call (~1-2 constants per call at 0.3).
pub const SUBTREE_STOP_PROB: f64 = 0.01;

/// In mutate_subtree at a binary node: which child to recurse into (left vs right).
/// Does NOT control whether mutation happens — SUBTREE_STOP_PROB handles that.
pub const BINARY_CHILD_SIDE_PROB: f64 = 0.005;


/// Range for randomised constant values.
pub const CONST_MUTATION_RANGE: (f32, f32) = (-1.0, 1.0);

/// Number of fresh-random individuals injected each generation.
/// 2 out of 16 = 12.5% diversity injection.
pub const FRESH_RANDOM_COUNT: usize = 1;

// ============================================================================

pub fn mutate(genome: &Genome, rng: &mut impl Rng) -> Genome {
    let mut tree = genome.tree();
    if rng.gen_bool(SUBTREE_MUTATION_PROB) {
        // 90%: fine-tuning — changes 1-2 constants, preserves structure
        tree = mutate_subtree(&tree, rng);
    } else {
        // 10%: more disruptive — can replace whole subtrees
        tree = replace_node(&tree, rng);
    }
    Genome::new(tree)
}

fn mutate_subtree(node: &Node, rng: &mut impl Rng) -> Node {
    // 30% chance to stop recursion here — keeps ~1-2 nodes changed per call
    if rng.gen_bool(SUBTREE_STOP_PROB) {
        return node.clone();
    }
    match node {
        Node::X | Node::Y => node.clone(),
        Node::Const(_) => Node::Const(rng.gen_range(CONST_MUTATION_RANGE.0..CONST_MUTATION_RANGE.1)),
        Node::Sin(child) => Node::Sin(Box::new(mutate_subtree(child, rng))),
        Node::Cos(child) => Node::Cos(Box::new(mutate_subtree(child, rng))),
        Node::Tan(child) => Node::Tan(Box::new(mutate_subtree(child, rng))),
        Node::Abs(child) => Node::Abs(Box::new(mutate_subtree(child, rng))),
        Node::Sqrt(child) => Node::Sqrt(Box::new(mutate_subtree(child, rng))),
        Node::Log(child) => Node::Log(Box::new(mutate_subtree(child, rng))),
        Node::Exp(child) => Node::Exp(Box::new(mutate_subtree(child, rng))),
        Node::Fract(child) => Node::Fract(Box::new(mutate_subtree(child, rng))),
        Node::Add(left, right) => {
            if rng.gen_bool(BINARY_CHILD_SIDE_PROB) {
                Node::Add(Box::new(mutate_subtree(left, rng)), Box::new(right.as_ref().clone()))
            } else {
                Node::Add(Box::new(left.as_ref().clone()), Box::new(mutate_subtree(right, rng)))
            }
        }
        Node::Sub(left, right) => {
            if rng.gen_bool(BINARY_CHILD_SIDE_PROB) {
                Node::Sub(Box::new(mutate_subtree(left, rng)), Box::new(right.as_ref().clone()))
            } else {
                Node::Sub(Box::new(left.as_ref().clone()), Box::new(mutate_subtree(right, rng)))
            }
        }
        Node::Mul(left, right) => {
            if rng.gen_bool(BINARY_CHILD_SIDE_PROB) {
                Node::Mul(Box::new(mutate_subtree(left, rng)), Box::new(right.as_ref().clone()))
            } else {
                Node::Mul(Box::new(left.as_ref().clone()), Box::new(mutate_subtree(right, rng)))
            }
        }
        Node::Div(left, right) => {
            if rng.gen_bool(BINARY_CHILD_SIDE_PROB) {
                Node::Div(Box::new(mutate_subtree(left, rng)), Box::new(right.as_ref().clone()))
            } else {
                Node::Div(Box::new(left.as_ref().clone()), Box::new(mutate_subtree(right, rng)))
            }
        }
        Node::Pow(left, right) => {
            if rng.gen_bool(BINARY_CHILD_SIDE_PROB) {
                Node::Pow(Box::new(mutate_subtree(left, rng)), Box::new(right.as_ref().clone()))
            } else {
                Node::Pow(Box::new(left.as_ref().clone()), Box::new(mutate_subtree(right, rng)))
            }
        }
        Node::Mix(low, high, t) => {
            if rng.gen_bool(BINARY_CHILD_SIDE_PROB) {
                Node::Mix(Box::new(mutate_subtree(low, rng)), Box::new(high.as_ref().clone()), Box::new(t.as_ref().clone()))
            } else {
                Node::Mix(Box::new(low.as_ref().clone()), Box::new(mutate_subtree(high, rng)), Box::new(t.as_ref().clone()))
            }
        }
        Node::Smoothstep(edge0, edge1, x) => {
            match rng.gen_range(0..3) {
                0 => Node::Smoothstep(Box::new(mutate_subtree(edge0, rng)), Box::new(edge1.as_ref().clone()), Box::new(x.as_ref().clone())),
                1 => Node::Smoothstep(Box::new(edge0.as_ref().clone()), Box::new(mutate_subtree(edge1, rng)), Box::new(x.as_ref().clone())),
                _ => Node::Smoothstep(Box::new(edge0.as_ref().clone()), Box::new(edge1.as_ref().clone()), Box::new(mutate_subtree(x, rng))),
            }
        }
        Node::Length(child) => Node::Length(Box::new(mutate_subtree(child, rng))),
        Node::Dot(left, right) => {
            if rng.gen_bool(BINARY_CHILD_SIDE_PROB) {
                Node::Dot(Box::new(mutate_subtree(left, rng)), Box::new(right.as_ref().clone()))
            } else {
                Node::Dot(Box::new(left.as_ref().clone()), Box::new(mutate_subtree(right, rng)))
            }
        }
    }
}

fn replace_node(node: &Node, rng: &mut impl Rng) -> Node {
    match node {
        Node::X | Node::Y => node.clone(),
        Node::Const(_) => Node::random(rng),
        Node::Sin(child) => Node::Sin(Box::new(replace_node(child, rng))),
        Node::Cos(child) => Node::Cos(Box::new(replace_node(child, rng))),
        Node::Tan(child) => Node::Tan(Box::new(replace_node(child, rng))),
        Node::Abs(child) => Node::Abs(Box::new(replace_node(child, rng))),
        Node::Sqrt(child) => Node::Sqrt(Box::new(replace_node(child, rng))),
        Node::Log(child) => Node::Log(Box::new(replace_node(child, rng))),
        Node::Exp(child) => Node::Exp(Box::new(replace_node(child, rng))),
        Node::Fract(child) => Node::Fract(Box::new(replace_node(child, rng))),
        Node::Add(left, right) => {
            match rng.gen_range(0..3) {
                0 => Node::Add(Box::new(replace_node(left, rng)), Box::new(right.as_ref().clone())),
                1 => Node::Add(Box::new(left.as_ref().clone()), Box::new(replace_node(right, rng))),
                _ => Node::random(rng),
            }
        }
        Node::Sub(left, right) => {
            match rng.gen_range(0..3) {
                0 => Node::Sub(Box::new(replace_node(left, rng)), Box::new(right.as_ref().clone())),
                1 => Node::Sub(Box::new(left.as_ref().clone()), Box::new(replace_node(right, rng))),
                _ => Node::random(rng),
            }
        }
        Node::Mul(left, right) => {
            match rng.gen_range(0..3) {
                0 => Node::Mul(Box::new(replace_node(left, rng)), Box::new(right.as_ref().clone())),
                1 => Node::Mul(Box::new(left.as_ref().clone()), Box::new(replace_node(right, rng))),
                _ => Node::random(rng),
            }
        }
        Node::Div(left, right) => {
            match rng.gen_range(0..3) {
                0 => Node::Div(Box::new(replace_node(left, rng)), Box::new(right.as_ref().clone())),
                1 => Node::Div(Box::new(left.as_ref().clone()), Box::new(replace_node(right, rng))),
                _ => Node::random(rng),
            }
        }
        Node::Pow(left, right) => {
            match rng.gen_range(0..3) {
                0 => Node::Pow(Box::new(replace_node(left, rng)), Box::new(right.as_ref().clone())),
                1 => Node::Pow(Box::new(left.as_ref().clone()), Box::new(replace_node(right, rng))),
                _ => Node::random(rng),
            }
        }
        Node::Mix(low, high, t) => {
            match rng.gen_range(0..3) {
                0 => Node::Mix(Box::new(replace_node(low, rng)), Box::new(high.as_ref().clone()), Box::new(t.as_ref().clone())),
                1 => Node::Mix(Box::new(low.as_ref().clone()), Box::new(replace_node(high, rng)), Box::new(t.as_ref().clone())),
                _ => Node::random(rng),
            }
        }
        Node::Smoothstep(edge0, edge1, x) => {
            match rng.gen_range(0..3) {
                0 => Node::Smoothstep(Box::new(replace_node(edge0, rng)), Box::new(edge1.as_ref().clone()), Box::new(x.as_ref().clone())),
                1 => Node::Smoothstep(Box::new(edge0.as_ref().clone()), Box::new(replace_node(edge1, rng)), Box::new(x.as_ref().clone())),
                _ => Node::Smoothstep(Box::new(edge0.as_ref().clone()), Box::new(edge1.as_ref().clone()), Box::new(replace_node(x, rng))),
            }
        }
        Node::Length(child) => Node::Length(Box::new(replace_node(child, rng))),
        Node::Dot(left, right) => {
            match rng.gen_range(0..3) {
                0 => Node::Dot(Box::new(replace_node(left, rng)), Box::new(right.as_ref().clone())),
                1 => Node::Dot(Box::new(left.as_ref().clone()), Box::new(replace_node(right, rng))),
                _ => Node::random(rng),
            }
        }
    }
}

// NOTE: crossover_subtree currently ignores `b` entirely — it always returns a
// structural copy of `a`. Result: crossover = random choice of one parent.
// This is intentionally conservative (children == one parent, no mixing).
// True subtree-swap crossover can be added later when visual exploration warrants it.
pub fn crossover(a: &Genome, b: &Genome, rng: &mut impl Rng) -> Genome {
    let tree = if rng.gen_bool(0.5) { a.tree() } else { b.tree() };
    Genome::new(tree)
}

pub fn selection(population: &[Genome], rng: &mut impl Rng) -> Genome {
    let size = population.len();
    if size == 1 {
        return population[0].clone();
    }
    let idx = rng.gen_range(0..size);
    population[idx].clone()
}

impl Genome {
    pub fn tree(&self) -> Node {
        instructions_to_tree(&self.instructions)
    }
}

fn instructions_to_tree(instructions: &[Instruction]) -> Node {
    let mut stack: Vec<Option<Node>> = Vec::new();

    // Find the last non-Const instruction (end of real computation)
    let real_end = instructions.iter().rposition(|i| i.op != OpCode::Const).unwrap_or(0);

    for instr in &instructions[..=real_end] {
        match instr.op {
            OpCode::X => {
                stack.push(Some(Node::X));
            }
            OpCode::Y => {
                stack.push(Some(Node::Y));
            }
            OpCode::Const => {
                stack.push(Some(Node::Const(instr.value)));
            }
            OpCode::Sin => {
                let idx = instr.a as usize;
                if idx < stack.len() {
                    if let Some(child) = stack[idx].clone() {
                        stack.push(Some(Node::Sin(Box::new(child))));
                    }
                }
            }
            OpCode::Cos => {
                let idx = instr.a as usize;
                if idx < stack.len() {
                    if let Some(child) = stack[idx].clone() {
                        stack.push(Some(Node::Cos(Box::new(child))));
                    }
                }
            }
            OpCode::Tan => {
                let idx = instr.a as usize;
                if idx < stack.len() {
                    if let Some(child) = stack[idx].clone() {
                        stack.push(Some(Node::Tan(Box::new(child))));
                    }
                }
            }
            OpCode::Abs => {
                let idx = instr.a as usize;
                if idx < stack.len() {
                    if let Some(child) = stack[idx].clone() {
                        stack.push(Some(Node::Abs(Box::new(child))));
                    }
                }
            }
            OpCode::Sqrt => {
                let idx = instr.a as usize;
                if idx < stack.len() {
                    if let Some(child) = stack[idx].clone() {
                        stack.push(Some(Node::Sqrt(Box::new(child))));
                    }
                }
            }
            OpCode::Log => {
                let idx = instr.a as usize;
                if idx < stack.len() {
                    if let Some(child) = stack[idx].clone() {
                        stack.push(Some(Node::Log(Box::new(child))));
                    }
                }
            }
            OpCode::Exp => {
                let idx = instr.a as usize;
                if idx < stack.len() {
                    if let Some(child) = stack[idx].clone() {
                        stack.push(Some(Node::Exp(Box::new(child))));
                    }
                }
            }
            OpCode::Fract => {
                let idx = instr.a as usize;
                if idx < stack.len() {
                    if let Some(child) = stack[idx].clone() {
                        stack.push(Some(Node::Fract(Box::new(child))));
                    }
                }
            }
            OpCode::Add => {
                let a = instr.a as usize;
                let b = instr.b as usize;
                if a < stack.len() && b < stack.len() {
                    if let (Some(left), Some(right)) = (stack[a].clone(), stack[b].clone()) {
                        stack.push(Some(Node::Add(Box::new(left), Box::new(right))));
                    }
                }
            }
            OpCode::Sub => {
                let a = instr.a as usize;
                let b = instr.b as usize;
                if a < stack.len() && b < stack.len() {
                    if let (Some(left), Some(right)) = (stack[a].clone(), stack[b].clone()) {
                        stack.push(Some(Node::Sub(Box::new(left), Box::new(right))));
                    }
                }
            }
            OpCode::Mul => {
                let a = instr.a as usize;
                let b = instr.b as usize;
                if a < stack.len() && b < stack.len() {
                    if let (Some(left), Some(right)) = (stack[a].clone(), stack[b].clone()) {
                        stack.push(Some(Node::Mul(Box::new(left), Box::new(right))));
                    }
                }
            }
            OpCode::Div => {
                let a = instr.a as usize;
                let b = instr.b as usize;
                if a < stack.len() && b < stack.len() {
                    if let (Some(left), Some(right)) = (stack[a].clone(), stack[b].clone()) {
                        stack.push(Some(Node::Div(Box::new(left), Box::new(right))));
                    }
                }
            }
            OpCode::Pow => {
                let a = instr.a as usize;
                let b = instr.b as usize;
                if a < stack.len() && b < stack.len() {
                    if let (Some(base), Some(exp)) = (stack[a].clone(), stack[b].clone()) {
                        stack.push(Some(Node::Pow(Box::new(base), Box::new(exp))));
                    }
                }
            }
            OpCode::Mix => {
                let a = instr.a as usize;
                let b = instr.b as usize;
                let c = instr.c as usize;
                if a < stack.len() && b < stack.len() && c < stack.len() {
                    if let (Some(low), Some(high), Some(t)) = (stack[a].clone(), stack[b].clone(), stack[c].clone()) {
                        stack.push(Some(Node::Mix(Box::new(low), Box::new(high), Box::new(t))));
                    }
                }
            }
            OpCode::Smoothstep => {
                let a = instr.a as usize;
                let b = instr.b as usize;
                let c = instr.c as usize;
                if a < stack.len() && b < stack.len() && c < stack.len() {
                    if let (Some(edge0), Some(edge1), Some(x)) = (stack[a].clone(), stack[b].clone(), stack[c].clone()) {
                        stack.push(Some(Node::Smoothstep(Box::new(edge0), Box::new(edge1), Box::new(x))));
                    }
                }
            }
            OpCode::Length => {
                let idx = instr.a as usize;
                if idx < stack.len() {
                    if let Some(child) = stack[idx].clone() {
                        stack.push(Some(Node::Length(Box::new(child))));
                    }
                }
            }
            OpCode::Dot => {
                let a = instr.a as usize;
                let b = instr.b as usize;
                if a < stack.len() && b < stack.len() {
                    if let (Some(left), Some(right)) = (stack[a].clone(), stack[b].clone()) {
                        stack.push(Some(Node::Dot(Box::new(left), Box::new(right))));
                    }
                }
            }
        }
    }

    stack.last().and_then(|n| n.clone()).unwrap_or(Node::Const(0.0))
}
