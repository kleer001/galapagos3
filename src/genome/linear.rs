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

// ============================================================================
// Eval helpers — one-line-per-op in Genome::eval
// ============================================================================

fn eval_unary(stack: &mut [f32], used: &mut usize, a: usize, f: impl Fn(f32) -> f32) {
    if a < *used {
        stack[*used] = f(stack[a]);
        *used += 1;
    }
}

fn eval_binary(stack: &mut [f32], used: &mut usize, a: usize, b: usize, f: impl Fn(f32, f32) -> f32) {
    if a < *used && b < *used {
        stack[*used] = f(stack[a], stack[b]);
        *used += 1;
    }
}

fn eval_ternary(
    stack: &mut [f32], used: &mut usize,
    a: usize, b: usize, c: usize,
    f: impl Fn(f32, f32, f32) -> f32,
) {
    if a < *used && b < *used && c < *used {
        stack[*used] = f(stack[a], stack[b], stack[c]);
        *used += 1;
    }
}

pub fn tree_to_instructions(tree: &Node) -> Vec<Instruction> {
    let mut stack = Vec::new();

    fn emit_unary(child: &Node, stack: &mut Vec<Instruction>, op: OpCode) {
        emit(child, stack);
        let idx = stack.len() as i32 - 1;
        stack.push(Instruction { op, a: idx, b: 0, c: 0, value: 0.0 });
    }

    fn emit_binary(left: &Node, right: &Node, stack: &mut Vec<Instruction>, op: OpCode) {
        emit(left, stack);
        let left_idx = stack.len() as i32 - 1;
        emit(right, stack);
        let right_idx = stack.len() as i32 - 1;
        stack.push(Instruction { op, a: left_idx, b: right_idx, c: 0, value: 0.0 });
    }

    fn emit_ternary(
        a: &Node, b: &Node, c: &Node,
        stack: &mut Vec<Instruction>, op: OpCode,
    ) {
        emit(a, stack);
        let a_idx = stack.len() as i32 - 1;
        emit(b, stack);
        let b_idx = stack.len() as i32 - 1;
        emit(c, stack);
        let c_idx = stack.len() as i32 - 1;
        stack.push(Instruction { op, a: a_idx, b: b_idx, c: c_idx, value: 0.0 });
    }

    fn emit(node: &Node, stack: &mut Vec<Instruction>) {
        match node {
            // Nullary
            Node::X       => stack.push(Instruction { op: OpCode::X, a: 0, b: 0, c: 0, value: 0.0 }),
            Node::Y       => stack.push(Instruction { op: OpCode::Y, a: 0, b: 0, c: 0, value: 0.0 }),
            Node::Const(v)=> stack.push(Instruction { op: OpCode::Const, a: 0, b: 0, c: 0, value: *v }),
            Node::MirrorX => stack.push(Instruction { op: OpCode::MirrorX, a: 0, b: 0, c: 0, value: 0.0 }),
            Node::MirrorY => stack.push(Instruction { op: OpCode::MirrorY, a: 0, b: 0, c: 0, value: 0.0 }),
            // Unary
            Node::Sin(c)        => emit_unary(c, stack, OpCode::Sin),
            Node::Cos(c)        => emit_unary(c, stack, OpCode::Cos),
            Node::Tan(c)        => emit_unary(c, stack, OpCode::Tan),
            Node::Abs(c)        => emit_unary(c, stack, OpCode::Abs),
            Node::Sqrt(c)       => emit_unary(c, stack, OpCode::Sqrt),
            Node::Log(c)        => emit_unary(c, stack, OpCode::Log),
            Node::Exp(c)        => emit_unary(c, stack, OpCode::Exp),
            Node::Fract(c)      => emit_unary(c, stack, OpCode::Fract),
            Node::Length(c)     => emit_unary(c, stack, OpCode::Length),
            Node::Acos(c)       => emit_unary(c, stack, OpCode::Acos),
            Node::Asin(c)       => emit_unary(c, stack, OpCode::Asin),
            Node::Atan(c)       => emit_unary(c, stack, OpCode::Atan),
            Node::Sinh(c)       => emit_unary(c, stack, OpCode::Sinh),
            Node::Cosh(c)       => emit_unary(c, stack, OpCode::Cosh),
            Node::Tanh(c)       => emit_unary(c, stack, OpCode::Tanh),
            Node::Sign(c)       => emit_unary(c, stack, OpCode::Sign),
            Node::Floor(c)      => emit_unary(c, stack, OpCode::Floor),
            Node::Ceil(c)       => emit_unary(c, stack, OpCode::Ceil),
            Node::Round(c)      => emit_unary(c, stack, OpCode::Round),
            Node::Negate(c)     => emit_unary(c, stack, OpCode::Negate),
            Node::Reciprocal(c) => emit_unary(c, stack, OpCode::Reciprocal),
            Node::Invert(c)     => emit_unary(c, stack, OpCode::Invert),
            // Binary
            Node::Add(l, r)        => emit_binary(l, r, stack, OpCode::Add),
            Node::Sub(l, r)        => emit_binary(l, r, stack, OpCode::Sub),
            Node::Mul(l, r)        => emit_binary(l, r, stack, OpCode::Mul),
            Node::Div(l, r)        => emit_binary(l, r, stack, OpCode::Div),
            Node::Pow(l, r)        => emit_binary(l, r, stack, OpCode::Pow),
            Node::Dot(l, r)        => emit_binary(l, r, stack, OpCode::Dot),
            Node::Min(l, r)        => emit_binary(l, r, stack, OpCode::Min),
            Node::Max(l, r)        => emit_binary(l, r, stack, OpCode::Max),
            Node::Step(l, r)       => emit_binary(l, r, stack, OpCode::Step),
            Node::ValueNoise(l, r) => emit_binary(l, r, stack, OpCode::ValueNoise),
            Node::WarpX(l, r)      => emit_binary(l, r, stack, OpCode::WarpX),
            Node::WarpY(l, r)      => emit_binary(l, r, stack, OpCode::WarpY),
            // Ternary
            Node::Mix(a, b, c)        => emit_ternary(a, b, c, stack, OpCode::Mix),
            Node::Smoothstep(a, b, c) => emit_ternary(a, b, c, stack, OpCode::Smoothstep),
            Node::Clamp(a, b, c)      => emit_ternary(a, b, c, stack, OpCode::Clamp),
            // Special: FBM stores octave count in c field (not a child index)
            Node::FBM(x, y, octaves) => {
                emit(x, stack);
                let x_idx = stack.len() as i32 - 1;
                emit(y, stack);
                let y_idx = stack.len() as i32 - 1;
                stack.push(Instruction { op: OpCode::FBM, a: x_idx, b: y_idx, c: *octaves, value: 0.0 });
            }
        }
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
            match instr.op {
                // Nullary
                OpCode::X       => { stack[used] = x; used += 1; }
                OpCode::Y       => { stack[used] = y; used += 1; }
                OpCode::Const   => { stack[used] = instr.value; used += 1; }
                OpCode::MirrorX => { stack[used] = x.abs(); used += 1; }
                OpCode::MirrorY => { stack[used] = y.abs(); used += 1; }
                // Unary
                OpCode::Sin    => eval_unary(&mut stack, &mut used, a, f32::sin),
                OpCode::Cos    => eval_unary(&mut stack, &mut used, a, f32::cos),
                OpCode::Tan    => eval_unary(&mut stack, &mut used, a, f32::tan),
                OpCode::Abs    => eval_unary(&mut stack, &mut used, a, f32::abs),
                OpCode::Sqrt   => eval_unary(&mut stack, &mut used, a, |v| v.sqrt().max(0.0)),
                OpCode::Log    => eval_unary(&mut stack, &mut used, a, f32::ln),
                OpCode::Exp    => eval_unary(&mut stack, &mut used, a, f32::exp),
                OpCode::Fract  => eval_unary(&mut stack, &mut used, a, f32::fract),
                OpCode::Length => eval_unary(&mut stack, &mut used, a, f32::abs),
                OpCode::Acos   => eval_unary(&mut stack, &mut used, a, |v| v.clamp(-1.0, 1.0).acos()),
                OpCode::Asin   => eval_unary(&mut stack, &mut used, a, |v| v.clamp(-1.0, 1.0).asin()),
                OpCode::Atan   => eval_unary(&mut stack, &mut used, a, f32::atan),
                OpCode::Sinh   => eval_unary(&mut stack, &mut used, a, f32::sinh),
                OpCode::Cosh   => eval_unary(&mut stack, &mut used, a, f32::cosh),
                OpCode::Tanh   => eval_unary(&mut stack, &mut used, a, f32::tanh),
                OpCode::Sign   => eval_unary(&mut stack, &mut used, a, |v| v.copysign(1.0)),
                OpCode::Floor  => eval_unary(&mut stack, &mut used, a, f32::floor),
                OpCode::Ceil   => eval_unary(&mut stack, &mut used, a, f32::ceil),
                OpCode::Round  => eval_unary(&mut stack, &mut used, a, f32::round),
                OpCode::Negate => eval_unary(&mut stack, &mut used, a, |v| -v),
                OpCode::Reciprocal => eval_unary(&mut stack, &mut used, a, |v| {
                    if v.abs() >= 1e-6 { 1.0 / v } else { 0.0 }
                }),
                OpCode::Invert => eval_unary(&mut stack, &mut used, a, |v| 1.0 - v),
                // Binary
                OpCode::Add   => eval_binary(&mut stack, &mut used, a, b, |x, y| x + y),
                OpCode::Sub   => eval_binary(&mut stack, &mut used, a, b, |x, y| x - y),
                OpCode::Mul   => eval_binary(&mut stack, &mut used, a, b, |x, y| x * y),
                OpCode::Div   => eval_binary(&mut stack, &mut used, a, b, |x, y| {
                    if y.abs() >= 1e-6 { x / y } else { 0.0 }
                }),
                OpCode::Pow   => eval_binary(&mut stack, &mut used, a, b, |x, y| {
                    if x > 0.0 { x.powf(y) } else { 0.0 }
                }),
                OpCode::Dot   => eval_binary(&mut stack, &mut used, a, b, |x, y| x * y),
                OpCode::Min   => eval_binary(&mut stack, &mut used, a, b, f32::min),
                OpCode::Max   => eval_binary(&mut stack, &mut used, a, b, f32::max),
                OpCode::Step  => eval_binary(&mut stack, &mut used, a, b, |edge, x| {
                    if x >= edge { 1.0 } else { 0.0 }
                }),
                OpCode::WarpX => eval_binary(&mut stack, &mut used, a, b, |x, y| x + y),
                OpCode::WarpY => eval_binary(&mut stack, &mut used, a, b, |x, y| x + y),
                // Ternary
                OpCode::Mix   => eval_ternary(&mut stack, &mut used, a, b, c, |lo, hi, t| lo + (hi - lo) * t),
                OpCode::Smoothstep => eval_ternary(&mut stack, &mut used, a, b, c, |e0, e1, x| {
                    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
                    t * (t - 2.0) * t * t + 1.0
                }),
                OpCode::Clamp => eval_ternary(&mut stack, &mut used, a, b, c, |v, lo, hi| v.clamp(lo, hi)),
                // Special: noise functions with custom logic
                OpCode::ValueNoise => {
                    if a < used && b < used {
                        let vx = stack[a];
                        let vy = stack[b];
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
                        stack[used] = lerp(top, bottom, fy);
                        used += 1;
                    }
                }
                OpCode::FBM => {
                    if a < used && b < used {
                        let mut value = 0.0;
                        let mut amplitude = 1.0;
                        let mut frequency = 1.0;
                        let mut max_val = 0.0;
                        for _ in 0..instr.c.clamp(1, 8) {
                            let vx = stack[a] * frequency;
                            let vy = stack[b] * frequency;
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
                            let noise = lerp(top, bottom, fy);
                            value += noise * amplitude;
                            max_val += amplitude;
                            amplitude *= 0.5;
                            frequency *= 2.0;
                        }
                        stack[used] = if max_val > 0.0 { value / max_val } else { 0.0 };
                        used += 1;
                    }
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
