use crate::config;
use crate::genome::linear::OpCode;
use crate::genome::op::{op_def, Arity, EvalFn, OP_REGISTRY};
use rand::Rng;

#[derive(Clone, Debug)]
pub struct Node {
    pub op: OpCode,
    pub children: Vec<Node>,
    pub value: f32,
    pub c_literal: i32,
}

impl Node {
    pub fn terminal(op: OpCode) -> Self {
        Node { op, children: vec![], value: 0.0, c_literal: 0 }
    }

    pub fn constant(v: f32) -> Self {
        Node { op: OpCode::Const, children: vec![], value: v, c_literal: 0 }
    }

    pub fn unary(op: OpCode, child: Node) -> Self {
        Node { op, children: vec![child], value: 0.0, c_literal: 0 }
    }

    pub fn binary(op: OpCode, left: Node, right: Node) -> Self {
        Node { op, children: vec![left, right], value: 0.0, c_literal: 0 }
    }

    pub fn ternary(op: OpCode, a: Node, b: Node, c: Node) -> Self {
        Node { op, children: vec![a, b, c], value: 0.0, c_literal: 0 }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        Self::random_with_depth(rng, config::MAX_TREE_DEPTH)
    }

    pub fn random_with_depth(rng: &mut impl Rng, max_depth: usize) -> Self {
        Self::random_bounded(rng, max_depth, config::MAX_TREE_SIZE)
    }

    fn random_terminal(rng: &mut impl Rng) -> Self {
        if rng.gen_bool(0.5) { Node::terminal(OpCode::X) } else { Node::terminal(OpCode::Y) }
    }

    fn random_bounded(rng: &mut impl Rng, max_depth: usize, max_size: usize) -> Self {
        if max_size < config::MIN_TREE_SIZE || max_depth == 0 {
            return Self::random_terminal(rng);
        }

        // 1% terminal rate
        if rng.gen_bool(0.01) {
            return Self::random_terminal(rng);
        }

        // Pick a non-terminal op from registry (skip X, Y, Const, PaletteT — PaletteT is only
        // meaningful in palette genomes where t holds the raw channel value)
        let eligible: Vec<&crate::genome::op::OpDef> = OP_REGISTRY.iter()
            .filter(|def| !matches!(def.opcode, OpCode::X | OpCode::Y | OpCode::Const | OpCode::PaletteT))
            .collect();
        let def = eligible[rng.gen_range(0..eligible.len())];

        match def.arity {
            Arity::Nullary => Node::terminal(def.opcode),
            Arity::Unary => {
                let child = Self::random_bounded(rng, max_depth - 1, max_size - 1);
                Node::unary(def.opcode, child)
            }
            Arity::Binary => {
                let budget = (max_size - 1) / 2;
                let a = Self::random_bounded(rng, max_depth - 1, budget);
                let b = Self::random_bounded(rng, max_depth - 1, budget);
                if def.opcode == OpCode::FBM {
                    let mut node = Node::binary(def.opcode, a, b);
                    node.c_literal = rng.gen_range(1..=6);
                    return node;
                }
                Node::binary(def.opcode, a, b)
            }
            Arity::Ternary => {
                let budget = (max_size - 1) / 3;
                let a = Self::random_bounded(rng, max_depth - 1, budget);
                let b = Self::random_bounded(rng, max_depth - 1, budget);
                // Mix: 3rd child is a constant blend parameter
                if def.opcode == OpCode::Mix {
                    let t = Node::constant(rng.gen_range(0.0..1.0));
                    return Node::ternary(def.opcode, a, b, t);
                }
                let c = Self::random_bounded(rng, max_depth - 1, budget);
                Node::ternary(def.opcode, a, b, c)
            }
        }
    }

    pub fn eval(&self, x: f32, y: f32, t: f32) -> f32 {
        let def = op_def(self.op);
        match &def.eval {
            EvalFn::PaletteTVal => t,
            EvalFn::Nullary(f) => f(x, y, self.value),
            EvalFn::Unary(f) => {
                let a = self.children.first().map_or(0.0, |c| c.eval(x, y, t));
                f(a)
            }
            EvalFn::Binary(f) => {
                let a = self.children.first().map_or(0.0, |c| c.eval(x, y, t));
                let b = self.children.get(1).map_or(0.0, |c| c.eval(x, y, t));
                f(a, b)
            }
            EvalFn::Ternary(f) => {
                let a = self.children.first().map_or(0.0, |c| c.eval(x, y, t));
                let b = self.children.get(1).map_or(0.0, |c| c.eval(x, y, t));
                let c = self.children.get(2).map_or(0.0, |c| c.eval(x, y, t));
                f(a, b, c)
            }
            EvalFn::BinaryLiteral(f) => {
                let a = self.children.first().map_or(0.0, |c| c.eval(x, y, t));
                let b = self.children.get(1).map_or(0.0, |c| c.eval(x, y, t));
                f(a, b, self.c_literal)
            }
        }
    }

    // ---- Palette genome generation (T is the only terminal) ----

    pub fn random_palette(rng: &mut impl Rng) -> Self {
        Self::random_palette_bounded(rng, config::MAX_TREE_DEPTH, config::MAX_TREE_SIZE)
    }

    pub fn random_palette_with_depth(rng: &mut impl Rng, max_depth: usize) -> Self {
        Self::random_palette_bounded(rng, max_depth, config::MAX_TREE_SIZE)
    }

    fn random_palette_terminal(_rng: &mut impl Rng) -> Self {
        Node::terminal(OpCode::PaletteT)
    }

    fn random_palette_bounded(rng: &mut impl Rng, max_depth: usize, max_size: usize) -> Self {
        if max_size < config::MIN_TREE_SIZE || max_depth == 0 {
            return Self::random_palette_terminal(rng);
        }
        if rng.gen_bool(0.01) {
            return Self::random_palette_terminal(rng);
        }

        // Same eligible ops as spatial (skip all spatial terminals and PaletteT at the
        // op-selection level; PaletteT will only appear as a leaf via random_palette_terminal)
        let eligible: Vec<&crate::genome::op::OpDef> = OP_REGISTRY.iter()
            .filter(|def| !matches!(def.opcode, OpCode::X | OpCode::Y | OpCode::MirrorX | OpCode::MirrorY | OpCode::Const | OpCode::PaletteT))
            .collect();
        let def = eligible[rng.gen_range(0..eligible.len())];

        match def.arity {
            Arity::Nullary => Node::terminal(def.opcode),
            Arity::Unary => {
                let child = Self::random_palette_bounded(rng, max_depth - 1, max_size - 1);
                Node::unary(def.opcode, child)
            }
            Arity::Binary => {
                let budget = (max_size - 1) / 2;
                let a = Self::random_palette_bounded(rng, max_depth - 1, budget);
                let b = Self::random_palette_bounded(rng, max_depth - 1, budget);
                if def.opcode == OpCode::FBM {
                    let mut node = Node::binary(def.opcode, a, b);
                    node.c_literal = rng.gen_range(1..=6);
                    return node;
                }
                Node::binary(def.opcode, a, b)
            }
            Arity::Ternary => {
                let budget = (max_size - 1) / 3;
                let a = Self::random_palette_bounded(rng, max_depth - 1, budget);
                let b = Self::random_palette_bounded(rng, max_depth - 1, budget);
                if def.opcode == OpCode::Mix {
                    let blend = Node::constant(rng.gen_range(0.0..1.0));
                    return Node::ternary(def.opcode, a, b, blend);
                }
                let c = Self::random_palette_bounded(rng, max_depth - 1, budget);
                Node::ternary(def.opcode, a, b, c)
            }
        }
    }
}
