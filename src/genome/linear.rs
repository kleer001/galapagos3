use crate::genome::Node;
use crate::genome::op::{op_def, EvalFn};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    X = 0,
    Y = 1,
    Const = 2,
    Sin = 3,
    Cos = 4,
    Tan = 5,
    Abs = 6,
    Sqrt = 7,
    Log = 8,
    Exp = 9,
    Fract = 10,
    Add = 11,
    Sub = 12,
    Mul = 13,
    Div = 14,
    Pow = 15,
    Mix = 16,
    Smoothstep = 17,
    Length = 18,
    Dot = 19,
    // Phase 2 operators
    Acos = 20,
    Asin = 21,
    Atan = 22,
    Sinh = 23,
    Cosh = 24,
    Tanh = 25,
    Min = 26,
    Max = 27,
    Clamp = 28,
    Sign = 29,
    Floor = 30,
    Ceil = 31,
    Round = 32,
    Negate = 33,
    Step = 34,
    Reciprocal = 35,
    Invert = 36,
    // Phase 3 operators
    ValueNoise = 37,
    FBM = 38,
    WarpX = 39,
    WarpY = 40,
    MirrorX = 41,
    MirrorY = 42,
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub op: OpCode,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub value: f32,
}

pub const MAX_INSTRUCTIONS: usize = 256;

pub fn tree_to_instructions(tree: &Node) -> Vec<Instruction> {
    let mut stack = Vec::new();

    fn emit(node: &Node, stack: &mut Vec<Instruction>) {
        let def = op_def(node.op);
        let count = def.arity.child_count();

        let mut child_indices = [0i32; 3];
        for (i, child) in node.children.iter().enumerate().take(count) {
            emit(child, stack);
            child_indices[i] = stack.len() as i32 - 1;
        }

        stack.push(Instruction {
            op: node.op,
            a: if count >= 1 { child_indices[0] } else { 0 },
            b: if count >= 2 { child_indices[1] } else { 0 },
            // Ternary: c = third child index. Otherwise: c = literal (e.g. FBM octaves).
            c: if count >= 3 { child_indices[2] } else { node.c_literal },
            value: node.value,
        });
    }

    emit(tree, &mut stack);

    let mut result: Vec<Instruction> = stack.into_iter().take(MAX_INSTRUCTIONS).collect();
    if result.len() < MAX_INSTRUCTIONS {
        let zero = Instruction { op: OpCode::Const, a: 0, b: 0, c: 0, value: 0.0 };
        result.extend(std::iter::repeat_n(zero, MAX_INSTRUCTIONS - result.len()));
    }
    result
}

#[derive(Debug, Clone)]
pub struct Genome {
    pub instructions: Vec<Instruction>,
}

impl Genome {
    pub fn new(tree: Node) -> Self {
        Genome {
            instructions: tree_to_instructions(&tree),
        }
    }

    pub fn eval(&self, x: f32, y: f32) -> f32 {
        let mut stack = vec![0.0; MAX_INSTRUCTIONS];
        let mut used = 0;

        let real_end = self.instructions.iter().rposition(|i| i.op != OpCode::Const)
            .unwrap_or(0);

        for i in 0..=real_end {
            let instr = &self.instructions[i];
            let (a, b, c) = (instr.a as usize, instr.b as usize, instr.c as usize);
            let def = op_def(instr.op);

            match &def.eval {
                EvalFn::Nullary(f) => {
                    stack[used] = f(x, y, instr.value);
                    used += 1;
                }
                EvalFn::Unary(f) => {
                    if a < used {
                        stack[used] = f(stack[a]);
                        used += 1;
                    }
                }
                EvalFn::Binary(f) => {
                    if a < used && b < used {
                        stack[used] = f(stack[a], stack[b]);
                        used += 1;
                    }
                }
                EvalFn::Ternary(f) => {
                    if a < used && b < used && c < used {
                        stack[used] = f(stack[a], stack[b], stack[c]);
                        used += 1;
                    }
                }
                EvalFn::BinaryLiteral(f) => {
                    if a < used && b < used {
                        stack[used] = f(stack[a], stack[b], instr.c);
                        used += 1;
                    }
                }
            }
        }

        if used > 0 { stack[used - 1] } else { 0.0 }
    }
}

#[derive(Debug, Clone)]
pub struct Population {
    pub genomes: Vec<Genome>,
}

impl Population {
    pub fn new(size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let genomes: Vec<Genome> = (0..size)
            .map(|_| Genome::new(Node::random(&mut rng)))
            .collect();
        Population { genomes }
    }
}
