use crate::config;
use crate::genome::{Genome, Instruction, OpCode, Node};
use crate::genome::op::{op_def, Arity};
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

fn mutate_subtree_with_params(node: &Node, rng: &mut impl Rng, params: &EvolutionParams) -> Node {
    if rng.gen_bool(params.subtree_stop_prob) {
        return node.clone();
    }

    let def = op_def(node.op);
    let mut result = node.clone();

    match def.arity {
        Arity::Nullary => {
            if node.op == OpCode::Const {
                result.value = rng.gen::<f32>();
            }
        }
        Arity::Unary => {
            result.children[0] = mutate_subtree_with_params(&node.children[0], rng, params);
        }
        Arity::Binary => {
            // FBM: 1/3 chance to mutate octaves instead of a child
            if node.op == OpCode::FBM {
                match rng.gen_range(0..3) {
                    0 => result.children[0] = mutate_subtree_with_params(&node.children[0], rng, params),
                    1 => result.children[1] = mutate_subtree_with_params(&node.children[1], rng, params),
                    _ => result.c_literal = rng.gen_range(1..=8),
                }
            } else if rng.gen_bool(params.binary_child_side_prob) {
                result.children[0] = mutate_subtree_with_params(&node.children[0], rng, params);
            } else {
                result.children[1] = mutate_subtree_with_params(&node.children[1], rng, params);
            }
        }
        Arity::Ternary => {
            // Mix: only mutate first two children (preserve blend parameter t)
            if node.op == OpCode::Mix {
                if rng.gen_bool(params.binary_child_side_prob) {
                    result.children[0] = mutate_subtree_with_params(&node.children[0], rng, params);
                } else {
                    result.children[1] = mutate_subtree_with_params(&node.children[1], rng, params);
                }
            } else {
                let idx = rng.gen_range(0..3);
                result.children[idx] = mutate_subtree_with_params(&node.children[idx], rng, params);
            }
        }
    }

    result
}

fn replace_node(node: &Node, rng: &mut impl Rng) -> Node {
    let def = op_def(node.op);

    match def.arity {
        Arity::Nullary => {
            if matches!(node.op, OpCode::X | OpCode::Y) {
                node.clone()
            } else {
                Node::random(rng)
            }
        }
        Arity::Unary => {
            let mut result = node.clone();
            result.children[0] = replace_node(&node.children[0], rng);
            result
        }
        Arity::Binary => {
            // 2/3 recurse into a child, 1/3 replace whole node
            match rng.gen_range(0..3) {
                0 => {
                    let mut result = node.clone();
                    result.children[0] = replace_node(&node.children[0], rng);
                    result
                }
                1 => {
                    let mut result = node.clone();
                    result.children[1] = replace_node(&node.children[1], rng);
                    result
                }
                _ => Node::random(rng),
            }
        }
        Arity::Ternary => {
            // Mix: only replace first two children or whole
            if node.op == OpCode::Mix {
                match rng.gen_range(0..3) {
                    0 => {
                        let mut result = node.clone();
                        result.children[0] = replace_node(&node.children[0], rng);
                        result
                    }
                    1 => {
                        let mut result = node.clone();
                        result.children[1] = replace_node(&node.children[1], rng);
                        result
                    }
                    _ => Node::random(rng),
                }
            } else {
                // 3/4 recurse into a child, 1/4 replace whole node
                match rng.gen_range(0..4) {
                    0 => {
                        let mut result = node.clone();
                        result.children[0] = replace_node(&node.children[0], rng);
                        result
                    }
                    1 => {
                        let mut result = node.clone();
                        result.children[1] = replace_node(&node.children[1], rng);
                        result
                    }
                    2 => {
                        let mut result = node.clone();
                        result.children[2] = replace_node(&node.children[2], rng);
                        result
                    }
                    _ => Node::random(rng),
                }
            }
        }
    }
}

// ---- Palette genome mutation (PaletteT is the "X/Y" of palette trees) ----

pub fn mutate_palette_with_params(genome: &Genome, rng: &mut impl Rng, params: &EvolutionParams) -> Genome {
    let mut candidate = genome.clone();
    for _ in 0..10 {
        let mut tree = genome.tree();
        if rng.gen_bool(params.subtree_mutation_prob) {
            tree = mutate_subtree_with_params(&tree, rng, params);
        } else {
            tree = replace_palette_node(&tree, rng);
        }
        candidate = Genome::new(tree);
        if candidate.palette_range() >= config::PALETTE_MIN_RANGE {
            return candidate;
        }
    }
    candidate
}

fn replace_palette_node(node: &Node, rng: &mut impl Rng) -> Node {
    let def = op_def(node.op);

    match def.arity {
        Arity::Nullary => {
            if node.op == OpCode::PaletteT {
                node.clone()
            } else {
                Node::random_palette(rng)
            }
        }
        Arity::Unary => {
            let mut result = node.clone();
            result.children[0] = replace_palette_node(&node.children[0], rng);
            result
        }
        Arity::Binary => {
            match rng.gen_range(0..3) {
                0 => {
                    let mut result = node.clone();
                    result.children[0] = replace_palette_node(&node.children[0], rng);
                    result
                }
                1 => {
                    let mut result = node.clone();
                    result.children[1] = replace_palette_node(&node.children[1], rng);
                    result
                }
                _ => Node::random_palette(rng),
            }
        }
        Arity::Ternary => {
            if node.op == OpCode::Mix {
                match rng.gen_range(0..3) {
                    0 => {
                        let mut result = node.clone();
                        result.children[0] = replace_palette_node(&node.children[0], rng);
                        result
                    }
                    1 => {
                        let mut result = node.clone();
                        result.children[1] = replace_palette_node(&node.children[1], rng);
                        result
                    }
                    _ => Node::random_palette(rng),
                }
            } else {
                match rng.gen_range(0..4) {
                    0 => {
                        let mut result = node.clone();
                        result.children[0] = replace_palette_node(&node.children[0], rng);
                        result
                    }
                    1 => {
                        let mut result = node.clone();
                        result.children[1] = replace_palette_node(&node.children[1], rng);
                        result
                    }
                    2 => {
                        let mut result = node.clone();
                        result.children[2] = replace_palette_node(&node.children[2], rng);
                        result
                    }
                    _ => Node::random_palette(rng),
                }
            }
        }
    }
}

pub fn crossover(a: &Genome, b: &Genome, rng: &mut impl Rng) -> Genome {
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

fn instructions_to_tree(instructions: &[Instruction]) -> Node {
    let mut stack: Vec<Option<Node>> = Vec::new();
    let real_end = instructions.iter().rposition(|i| i.op != OpCode::Const).unwrap_or(0);

    for instr in &instructions[..=real_end] {
        let def = op_def(instr.op);
        let count = def.arity.child_count();
        let indices = [instr.a as usize, instr.b as usize, instr.c as usize];

        // Collect children from stack
        let mut children = Vec::with_capacity(count);
        let mut valid = true;
        for &idx in &indices[..count] {
            if idx < stack.len() {
                if let Some(child) = stack[idx].clone() {
                    children.push(child);
                } else {
                    valid = false;
                    break;
                }
            } else {
                valid = false;
                break;
            }
        }

        if valid {
            stack.push(Some(Node {
                op: instr.op,
                children,
                value: instr.value,
                c_literal: instr.c,
            }));
        }
    }

    stack.last().and_then(|n| n.clone()).unwrap_or(Node::constant(0.0))
}
