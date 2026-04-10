use crate::genome::Node;

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
    Radial = 37,
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub op: OpCode,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub value: f32,
}

pub const MAX_INSTRUCTIONS: usize = 64;

pub fn tree_to_instructions(tree: &Node) -> Vec<Instruction> {
    let mut stack = Vec::new();

    fn emit(node: &Node, stack: &mut Vec<Instruction>) {
        match node {
            Node::X => {
                stack.push(Instruction { op: OpCode::X, a: 0, b: 0, c: 0, value: 0.0 });
            }
            Node::Y => {
                stack.push(Instruction { op: OpCode::Y, a: 0, b: 0, c: 0, value: 0.0 });
            }
            Node::Const(v) => {
                stack.push(Instruction { op: OpCode::Const, a: 0, b: 0, c: 0, value: *v });
            }
            Node::Sin(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Sin, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Cos(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Cos, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Tan(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Tan, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Abs(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Abs, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Sqrt(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Sqrt, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Log(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Log, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Exp(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Exp, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Fract(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Fract, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Add(left, right) => {
                emit(left, stack);
                let left_idx = stack.len() as i32 - 1;
                emit(right, stack);
                let right_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Add, a: left_idx, b: right_idx, c: 0, value: 0.0 });
            }
            Node::Sub(left, right) => {
                emit(left, stack);
                let left_idx = stack.len() as i32 - 1;
                emit(right, stack);
                let right_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Sub, a: left_idx, b: right_idx, c: 0, value: 0.0 });
            }
            Node::Mul(left, right) => {
                emit(left, stack);
                let left_idx = stack.len() as i32 - 1;
                emit(right, stack);
                let right_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Mul, a: left_idx, b: right_idx, c: 0, value: 0.0 });
            }
            Node::Div(left, right) => {
                emit(left, stack);
                let left_idx = stack.len() as i32 - 1;
                emit(right, stack);
                let right_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Div, a: left_idx, b: right_idx, c: 0, value: 0.0 });
            }
            Node::Pow(base, exp) => {
                emit(base, stack);
                let base_idx = stack.len() as i32 - 1;
                emit(exp, stack);
                let exp_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Pow, a: base_idx, b: exp_idx, c: 0, value: 0.0 });
            }
            Node::Mix(low, high, t) => {
                emit(low, stack);
                let low_idx = stack.len() as i32 - 1;
                emit(high, stack);
                let high_idx = stack.len() as i32 - 1;
                emit(t, stack);
                let t_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Mix, a: low_idx, b: high_idx, c: t_idx, value: 0.0 });
            }
            Node::Smoothstep(edge0, edge1, x) => {
                emit(edge0, stack);
                let edge0_idx = stack.len() as i32 - 1;
                emit(edge1, stack);
                let edge1_idx = stack.len() as i32 - 1;
                emit(x, stack);
                let x_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Smoothstep, a: edge0_idx, b: edge1_idx, c: x_idx, value: 0.0 });
            }
            Node::Length(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Length, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Dot(left, right) => {
                emit(left, stack);
                let left_idx = stack.len() as i32 - 1;
                emit(right, stack);
                let right_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Dot, a: left_idx, b: right_idx, c: 0, value: 0.0 });
            }
            // Phase 2 operators
            Node::Acos(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Acos, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Asin(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Asin, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Atan(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Atan, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Sinh(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Sinh, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Cosh(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Cosh, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Tanh(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Tanh, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Min(left, right) => {
                emit(left, stack);
                let left_idx = stack.len() as i32 - 1;
                emit(right, stack);
                let right_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Min, a: left_idx, b: right_idx, c: 0, value: 0.0 });
            }
            Node::Max(left, right) => {
                emit(left, stack);
                let left_idx = stack.len() as i32 - 1;
                emit(right, stack);
                let right_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Max, a: left_idx, b: right_idx, c: 0, value: 0.0 });
            }
            Node::Clamp(value, min, max) => {
                emit(value, stack);
                let value_idx = stack.len() as i32 - 1;
                emit(min, stack);
                let min_idx = stack.len() as i32 - 1;
                emit(max, stack);
                let max_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Clamp, a: value_idx, b: min_idx, c: max_idx, value: 0.0 });
            }
            Node::Sign(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Sign, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Floor(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Floor, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Ceil(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Ceil, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Round(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Round, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Negate(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Negate, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Step(edge, x_node) => {
                emit(edge, stack);
                let edge_idx = stack.len() as i32 - 1;
                emit(x_node, stack);
                let x_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Step, a: edge_idx, b: x_idx, c: 0, value: 0.0 });
            }
            Node::Reciprocal(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Reciprocal, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Invert(child) => {
                emit(child, stack);
                let child_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::Invert, a: child_idx, b: 0, c: 0, value: 0.0 });
            }
            Node::Radial => {
                // Radial computes sqrt(x*x + y*y) where x and y are the coordinates
                stack.push(Instruction { op: OpCode::Radial, a: 0, b: 0, c: 0, value: 0.0 });
            }
        }
    }

    emit(tree, &mut stack);

    let mut result: Vec<Instruction> = stack.into_iter().take(MAX_INSTRUCTIONS).collect();
    if result.len() < MAX_INSTRUCTIONS {
        let zero = Instruction { op: OpCode::Const, a: 0, b: 0, c: 0, value: 0.0 };
        result.extend(std::iter::repeat(zero).take(MAX_INSTRUCTIONS - result.len()));
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

        // Find the last non-Const instruction (end of real computation)
        let real_end = self.instructions.iter().rposition(|i| i.op != OpCode::Const)
            .unwrap_or(0);
        for i in 0..=real_end {
            let instr = &self.instructions[i];
            match instr.op {
                OpCode::X => {
                    stack[used] = x;
                    used += 1;
                }
                OpCode::Y => {
                    stack[used] = y;
                    used += 1;
                }
                OpCode::Const => {
                    stack[used] = instr.value;
                    used += 1;
                }
                OpCode::Sin => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].sin();
                        used += 1;
                    }
                }
                OpCode::Cos => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].cos();
                        used += 1;
                    }
                }
                OpCode::Tan => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].tan();
                        used += 1;
                    }
                }
                OpCode::Abs => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].abs();
                        used += 1;
                    }
                }
                OpCode::Sqrt => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].sqrt().max(0.0);
                        used += 1;
                    }
                }
                OpCode::Log => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].ln();
                        used += 1;
                    }
                }
                OpCode::Exp => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].exp();
                        used += 1;
                    }
                }
                OpCode::Fract => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].fract();
                        used += 1;
                    }
                }
                OpCode::Add => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    if a < used && b < used {
                        stack[used] = stack[a] + stack[b];
                        used += 1;
                    }
                }
                OpCode::Sub => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    if a < used && b < used {
                        stack[used] = stack[a] - stack[b];
                        used += 1;
                    }
                }
                OpCode::Mul => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    if a < used && b < used {
                        stack[used] = stack[a] * stack[b];
                        used += 1;
                    }
                }
                OpCode::Div => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    if a < used && b < used {
                        let denom = stack[b];
                        stack[used] = if denom.abs() >= 1e-6 { stack[a] / denom } else { 0.0 };
                        used += 1;
                    }
                }
                OpCode::Pow => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    if a < used && b < used {
                        let base = stack[a];
                        let exp = stack[b];
                        stack[used] = if base > 0.0 { base.powf(exp) } else { 0.0 };
                        used += 1;
                    }
                }
                OpCode::Mix => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    let c = instr.c as usize;
                    if a < used && b < used && c < used {
                        let t = stack[c];
                        stack[used] = stack[a] + (stack[b] - stack[a]) * t;
                        used += 1;
                    }
                }
                OpCode::Smoothstep => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    let c = instr.c as usize;
                    if a < used && b < used && c < used {
                        let edge0 = stack[a];
                        let edge1 = stack[b];
                        let x_val = stack[c];
                        let t = (x_val - edge0) / (edge1 - edge0);
                        let t_clamped = t.clamp(0.0, 1.0);
                        stack[used] = t_clamped * (t_clamped - 2.0) * t_clamped * t_clamped + 1.0;
                        used += 1;
                    }
                }
                OpCode::Length => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].abs();
                        used += 1;
                    }
                }
                OpCode::Dot => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    if a < used && b < used {
                        stack[used] = stack[a] * stack[b];
                        used += 1;
                    }
                }
                // Phase 2 operators
                OpCode::Acos => {
                    let idx = instr.a as usize;
                    if idx < used {
                        let v = stack[idx].clamp(-1.0, 1.0);
                        stack[used] = v.acos();
                        used += 1;
                    }
                }
                OpCode::Asin => {
                    let idx = instr.a as usize;
                    if idx < used {
                        let v = stack[idx].clamp(-1.0, 1.0);
                        stack[used] = v.asin();
                        used += 1;
                    }
                }
                OpCode::Atan => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].atan();
                        used += 1;
                    }
                }
                OpCode::Sinh => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].sinh();
                        used += 1;
                    }
                }
                OpCode::Cosh => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].cosh();
                        used += 1;
                    }
                }
                OpCode::Tanh => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].tanh();
                        used += 1;
                    }
                }
                OpCode::Min => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    if a < used && b < used {
                        stack[used] = stack[a].min(stack[b]);
                        used += 1;
                    }
                }
                OpCode::Max => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    if a < used && b < used {
                        stack[used] = stack[a].max(stack[b]);
                        used += 1;
                    }
                }
                OpCode::Clamp => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    let c = instr.c as usize;
                    if a < used && b < used && c < used {
                        let v = stack[a];
                        let lo = stack[b];
                        let hi = stack[c];
                        stack[used] = v.clamp(lo, hi);
                        used += 1;
                    }
                }
                OpCode::Sign => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].copysign(1.0);
                        used += 1;
                    }
                }
                OpCode::Floor => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].floor();
                        used += 1;
                    }
                }
                OpCode::Ceil => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].ceil();
                        used += 1;
                    }
                }
                OpCode::Round => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = stack[idx].round();
                        used += 1;
                    }
                }
                OpCode::Negate => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = -stack[idx];
                        used += 1;
                    }
                }
                OpCode::Step => {
                    let a = instr.a as usize;
                    let b = instr.b as usize;
                    if a < used && b < used {
                        let edge = stack[a];
                        let x_val = stack[b];
                        stack[used] = if x_val >= edge { 1.0 } else { 0.0 };
                        used += 1;
                    }
                }
                OpCode::Reciprocal => {
                    let idx = instr.a as usize;
                    if idx < used {
                        let v = stack[idx];
                        stack[used] = if v.abs() >= 1e-6 { 1.0 / v } else { 0.0 };
                        used += 1;
                    }
                }
                OpCode::Invert => {
                    let idx = instr.a as usize;
                    if idx < used {
                        stack[used] = 1.0 - stack[idx];
                        used += 1;
                    }
                }
                OpCode::Radial => {
                    stack[used] = (x * x + y * y).sqrt();
                    used += 1;
                }
            }
        }

        if used > 0 {
            stack[used - 1]
        } else {
            0.0
        }
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
