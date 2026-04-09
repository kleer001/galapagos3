use crate::genome::{Genome, Instruction, OpCode, Node};
use rand::Rng;

pub fn mutate(genome: &Genome, rng: &mut impl Rng) -> Genome {
    let mut tree = genome.tree();
    if rng.gen_bool(0.5) {
        tree = mutate_subtree(&tree, rng);
    } else {
        tree = replace_node(&tree, rng);
    }
    Genome::new(tree)
}

fn mutate_subtree(node: &Node, rng: &mut impl Rng) -> Node {
    match node {
        Node::X | Node::Y => node.clone(),
        Node::Const(_) => Node::Const(rng.gen_range(-10.0..10.0)),
        Node::Sin(child) => Node::Sin(Box::new(mutate_subtree(child, rng))),
        Node::Cos(child) => Node::Cos(Box::new(mutate_subtree(child, rng))),
        Node::Tan(child) => Node::Tan(Box::new(mutate_subtree(child, rng))),
        Node::Abs(child) => Node::Abs(Box::new(mutate_subtree(child, rng))),
        Node::Sqrt(child) => Node::Sqrt(Box::new(mutate_subtree(child, rng))),
        Node::Log(child) => Node::Log(Box::new(mutate_subtree(child, rng))),
        Node::Exp(child) => Node::Exp(Box::new(mutate_subtree(child, rng))),
        Node::Fract(child) => Node::Fract(Box::new(mutate_subtree(child, rng))),
        Node::Add(left, right) => {
            if rng.gen_bool(0.5) {
                Node::Add(Box::new(mutate_subtree(left, rng)), Box::new(right.as_ref().clone()))
            } else {
                Node::Add(Box::new(left.as_ref().clone()), Box::new(mutate_subtree(right, rng)))
            }
        }
        Node::Sub(left, right) => {
            if rng.gen_bool(0.5) {
                Node::Sub(Box::new(mutate_subtree(left, rng)), Box::new(right.as_ref().clone()))
            } else {
                Node::Sub(Box::new(left.as_ref().clone()), Box::new(mutate_subtree(right, rng)))
            }
        }
        Node::Mul(left, right) => {
            if rng.gen_bool(0.5) {
                Node::Mul(Box::new(mutate_subtree(left, rng)), Box::new(right.as_ref().clone()))
            } else {
                Node::Mul(Box::new(left.as_ref().clone()), Box::new(mutate_subtree(right, rng)))
            }
        }
        Node::Div(left, right) => {
            if rng.gen_bool(0.5) {
                Node::Div(Box::new(mutate_subtree(left, rng)), Box::new(right.as_ref().clone()))
            } else {
                Node::Div(Box::new(left.as_ref().clone()), Box::new(mutate_subtree(right, rng)))
            }
        }
        Node::Pow(left, right) => {
            if rng.gen_bool(0.5) {
                Node::Pow(Box::new(mutate_subtree(left, rng)), Box::new(right.as_ref().clone()))
            } else {
                Node::Pow(Box::new(left.as_ref().clone()), Box::new(mutate_subtree(right, rng)))
            }
        }
        Node::Mix(low, high, t) => {
            if rng.gen_bool(0.5) {
                Node::Mix(Box::new(mutate_subtree(low, rng)), Box::new(high.as_ref().clone()), Box::new(t.as_ref().clone()))
            } else {
                Node::Mix(Box::new(low.as_ref().clone()), Box::new(mutate_subtree(high, rng)), Box::new(t.as_ref().clone()))
            }
        }
        Node::Smoothstep(edge0, edge1, x) => {
            if rng.gen_bool(0.33) {
                Node::Smoothstep(Box::new(mutate_subtree(edge0, rng)), Box::new(edge1.as_ref().clone()), Box::new(x.as_ref().clone()))
            } else if rng.gen_bool(0.33) {
                Node::Smoothstep(Box::new(edge0.as_ref().clone()), Box::new(mutate_subtree(edge1, rng)), Box::new(x.as_ref().clone()))
            } else {
                Node::Smoothstep(Box::new(edge0.as_ref().clone()), Box::new(edge1.as_ref().clone()), Box::new(mutate_subtree(x, rng)))
            }
        }
        Node::Length(child) => Node::Length(Box::new(mutate_subtree(child, rng))),
        Node::Dot(left, right) => {
            if rng.gen_bool(0.5) {
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

pub fn crossover(a: &Genome, b: &Genome, rng: &mut impl Rng) -> Genome {
    let tree_a = a.tree();
    let tree_b = b.tree();

    let child_tree = match rng.gen_range(0..2) {
        0 => crossover_subtree(&tree_a, &tree_b, rng),
        _ => crossover_subtree(&tree_b, &tree_a, rng),
    };

    Genome::new(child_tree)
}

fn crossover_subtree(a: &Node, b: &Node, rng: &mut impl Rng) -> Node {
    match a {
        Node::X | Node::Y => a.clone(),
        Node::Const(_) => a.clone(),
        Node::Sin(child_a) => Node::Sin(Box::new(crossover_subtree(child_a, b, rng))),
        Node::Cos(child_a) => Node::Cos(Box::new(crossover_subtree(child_a, b, rng))),
        Node::Tan(child_a) => Node::Tan(Box::new(crossover_subtree(child_a, b, rng))),
        Node::Abs(child_a) => Node::Abs(Box::new(crossover_subtree(child_a, b, rng))),
        Node::Sqrt(child_a) => Node::Sqrt(Box::new(crossover_subtree(child_a, b, rng))),
        Node::Log(child_a) => Node::Log(Box::new(crossover_subtree(child_a, b, rng))),
        Node::Exp(child_a) => Node::Exp(Box::new(crossover_subtree(child_a, b, rng))),
        Node::Fract(child_a) => Node::Fract(Box::new(crossover_subtree(child_a, b, rng))),
        Node::Add(left_a, right_a) => {
            let left = if rng.gen_bool(0.5) {
                crossover_subtree(left_a, b, rng)
            } else {
                left_a.as_ref().clone()
            };
            let right = if rng.gen_bool(0.5) {
                crossover_subtree(right_a, b, rng)
            } else {
                right_a.as_ref().clone()
            };
            Node::Add(Box::new(left), Box::new(right))
        }
        Node::Sub(left_a, right_a) => {
            let left = if rng.gen_bool(0.5) {
                crossover_subtree(left_a, b, rng)
            } else {
                left_a.as_ref().clone()
            };
            let right = if rng.gen_bool(0.5) {
                crossover_subtree(right_a, b, rng)
            } else {
                right_a.as_ref().clone()
            };
            Node::Sub(Box::new(left), Box::new(right))
        }
        Node::Mul(left_a, right_a) => {
            let left = if rng.gen_bool(0.5) {
                crossover_subtree(left_a, b, rng)
            } else {
                left_a.as_ref().clone()
            };
            let right = if rng.gen_bool(0.5) {
                crossover_subtree(right_a, b, rng)
            } else {
                right_a.as_ref().clone()
            };
            Node::Mul(Box::new(left), Box::new(right))
        }
        Node::Div(left_a, right_a) => {
            let left = if rng.gen_bool(0.5) {
                crossover_subtree(left_a, b, rng)
            } else {
                left_a.as_ref().clone()
            };
            let right = if rng.gen_bool(0.5) {
                crossover_subtree(right_a, b, rng)
            } else {
                right_a.as_ref().clone()
            };
            Node::Div(Box::new(left), Box::new(right))
        }
        Node::Pow(left_a, right_a) => {
            let left = if rng.gen_bool(0.5) {
                crossover_subtree(left_a, b, rng)
            } else {
                left_a.as_ref().clone()
            };
            let right = if rng.gen_bool(0.5) {
                crossover_subtree(right_a, b, rng)
            } else {
                right_a.as_ref().clone()
            };
            Node::Pow(Box::new(left), Box::new(right))
        }
        Node::Mix(low_a, high_a, t_a) => {
            let low = if rng.gen_bool(0.5) {
                crossover_subtree(low_a, b, rng)
            } else {
                low_a.as_ref().clone()
            };
            let high = if rng.gen_bool(0.5) {
                crossover_subtree(high_a, b, rng)
            } else {
                high_a.as_ref().clone()
            };
            let t = if rng.gen_bool(0.5) {
                crossover_subtree(t_a, b, rng)
            } else {
                t_a.as_ref().clone()
            };
            Node::Mix(Box::new(low), Box::new(high), Box::new(t))
        }
        Node::Smoothstep(edge0_a, edge1_a, x_a) => {
            let edge0 = if rng.gen_bool(0.5) {
                crossover_subtree(edge0_a, b, rng)
            } else {
                edge0_a.as_ref().clone()
            };
            let edge1 = if rng.gen_bool(0.5) {
                crossover_subtree(edge1_a, b, rng)
            } else {
                edge1_a.as_ref().clone()
            };
            let x = if rng.gen_bool(0.5) {
                crossover_subtree(x_a, b, rng)
            } else {
                x_a.as_ref().clone()
            };
            Node::Smoothstep(Box::new(edge0), Box::new(edge1), Box::new(x))
        }
        Node::Length(child_a) => Node::Length(Box::new(crossover_subtree(child_a, b, rng))),
        Node::Dot(left_a, right_a) => {
            let left = if rng.gen_bool(0.5) {
                crossover_subtree(left_a, b, rng)
            } else {
                left_a.as_ref().clone()
            };
            let right = if rng.gen_bool(0.5) {
                crossover_subtree(right_a, b, rng)
            } else {
                right_a.as_ref().clone()
            };
            Node::Dot(Box::new(left), Box::new(right))
        }
    }
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
    let mut stack = Vec::new();

    for instr in instructions {
        match instr.op {
            OpCode::X => {
                stack.push(Node::X);
            }
            OpCode::Y => {
                stack.push(Node::Y);
            }
            OpCode::Const => {
                stack.push(Node::Const(instr.value));
            }
            OpCode::Sin => {
                if let Some(child) = stack.pop() {
                    stack.push(Node::Sin(Box::new(child)));
                }
            }
            OpCode::Cos => {
                if let Some(child) = stack.pop() {
                    stack.push(Node::Cos(Box::new(child)));
                }
            }
            OpCode::Tan => {
                if let Some(child) = stack.pop() {
                    stack.push(Node::Tan(Box::new(child)));
                }
            }
            OpCode::Abs => {
                if let Some(child) = stack.pop() {
                    stack.push(Node::Abs(Box::new(child)));
                }
            }
            OpCode::Sqrt => {
                if let Some(child) = stack.pop() {
                    stack.push(Node::Sqrt(Box::new(child)));
                }
            }
            OpCode::Log => {
                if let Some(child) = stack.pop() {
                    stack.push(Node::Log(Box::new(child)));
                }
            }
            OpCode::Exp => {
                if let Some(child) = stack.pop() {
                    stack.push(Node::Exp(Box::new(child)));
                }
            }
            OpCode::Fract => {
                if let Some(child) = stack.pop() {
                    stack.push(Node::Fract(Box::new(child)));
                }
            }
            OpCode::Add => {
                if let (Some(left), Some(right)) = (stack.pop(), stack.pop()) {
                    stack.push(Node::Add(Box::new(left), Box::new(right)));
                }
            }
            OpCode::Sub => {
                if let (Some(left), Some(right)) = (stack.pop(), stack.pop()) {
                    stack.push(Node::Sub(Box::new(left), Box::new(right)));
                }
            }
            OpCode::Mul => {
                if let (Some(left), Some(right)) = (stack.pop(), stack.pop()) {
                    stack.push(Node::Mul(Box::new(left), Box::new(right)));
                }
            }
            OpCode::Div => {
                if let (Some(left), Some(right)) = (stack.pop(), stack.pop()) {
                    stack.push(Node::Div(Box::new(left), Box::new(right)));
                }
            }
            OpCode::Pow => {
                if let (Some(left), Some(right)) = (stack.pop(), stack.pop()) {
                    stack.push(Node::Pow(Box::new(left), Box::new(right)));
                }
            }
            OpCode::Mix => {
                if let (Some(low), Some(high), Some(t)) = (stack.pop(), stack.pop(), stack.pop()) {
                    stack.push(Node::Mix(Box::new(low), Box::new(high), Box::new(t)));
                }
            }
            OpCode::Smoothstep => {
                if let (Some(edge0), Some(edge1), Some(x)) = (stack.pop(), stack.pop(), stack.pop()) {
                    stack.push(Node::Smoothstep(Box::new(edge0), Box::new(edge1), Box::new(x)));
                }
            }
            OpCode::Length => {
                if let Some(child) = stack.pop() {
                    stack.push(Node::Length(Box::new(child)));
                }
            }
            OpCode::Dot => {
                if let (Some(left), Some(right)) = (stack.pop(), stack.pop()) {
                    stack.push(Node::Dot(Box::new(left), Box::new(right)));
                }
            }
        }
    }

    stack.pop().unwrap_or(Node::Const(0.0))
}
