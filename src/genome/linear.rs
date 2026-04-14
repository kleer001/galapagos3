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
    // Phase 2 operators
    Acos = 18,
    Asin = 19,
    Atan = 20,
    Sinh = 21,
    Cosh = 22,
    Tanh = 23,
    Min = 24,
    Max = 25,
    Clamp = 26,
    Sign = 27,
    Floor = 28,
    Ceil = 29,
    Round = 30,
    Negate = 31,
    Step = 32,
    Reciprocal = 33,
    Invert = 34,
    // Phase 3 operators
    ValueNoise = 35,
    FBM = 36,
    MirrorX = 37,
    MirrorY = 38,
    // New operators
    Atan2 = 39,
    Mod = 40,
    Worley = 41,
    TriWave = 42,
    Chebyshev = 43,
    Manhattan = 44,
    SinFold = 45,
    PaletteT = 46,
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub op: OpCode,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub value: f32,
}

pub const MAX_INSTRUCTIONS: usize = 1024;

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

    /// Reconstruct a human-readable expression string from the flat instruction list.
    /// `t_expr` is substituted wherever a PaletteT node appears; pass `"t"` for the default.
    pub fn to_expr_string_with_t(&self, t_expr: &str) -> String {
        let real_end = self.instructions.iter().rposition(|i| i.op != OpCode::Const)
            .unwrap_or(0);
        let mut exprs: Vec<String> = Vec::with_capacity(real_end + 1);

        for k in 0..=real_end {
            let instr = &self.instructions[k];
            let (a, b, c) = (instr.a as usize, instr.b as usize, instr.c as usize);
            let def = op_def(instr.op);
            let name = def.name.to_lowercase();
            let get = |idx: usize| exprs.get(idx).map(|s| s.as_str()).unwrap_or("?");

            let s = match &def.eval {
                EvalFn::PaletteTVal => t_expr.to_string(),
                EvalFn::Nullary(_) => {
                    if instr.op == OpCode::Const {
                        format!("{:.3}", instr.value)
                    } else {
                        name
                    }
                }
                EvalFn::Unary(_) => format!("{}({})", name, get(a)),
                EvalFn::Binary(_) => format!("{}({}, {})", name, get(a), get(b)),
                EvalFn::Ternary(_) => format!("{}({}, {}, {})", name, get(a), get(b), get(c)),
                EvalFn::BinaryLiteral(_) => format!("{}({}, {}, {})", name, get(a), get(b), instr.c),
            };
            exprs.push(s);
        }

        exprs.into_iter().last().unwrap_or_else(|| "0.000".to_string())
    }

    pub fn to_expr_string(&self) -> String {
        self.to_expr_string_with_t("t")
    }

    pub fn eval(&self, x: f32, y: f32, t: f32) -> f32 {
        let mut stack = vec![0.0; MAX_INSTRUCTIONS];
        let mut used = 0;

        let real_end = self.instructions.iter().rposition(|i| i.op != OpCode::Const)
            .unwrap_or(0);

        for i in 0..=real_end {
            let instr = &self.instructions[i];
            let (a, b, c) = (instr.a as usize, instr.b as usize, instr.c as usize);
            let def = op_def(instr.op);

            match &def.eval {
                EvalFn::PaletteTVal => {
                    stack[used] = t;
                    used += 1;
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::Node;

    fn spatial_terminals(expr: &str) -> bool {
        expr.contains('x') || expr.contains('y')
    }

    fn has_palette_t(expr: &str) -> bool {
        // "t" appears as a standalone token (not inside a longer name like "atan2")
        expr.split(|c: char| !c.is_alphanumeric() && c != '_')
            .any(|tok| tok == "t")
    }

    fn no_spatial_terminals(expr: &str) -> bool {
        // ensure neither bare "x" nor bare "y" appear as tokens
        !expr.split(|c: char| !c.is_alphanumeric() && c != '_')
            .any(|tok| tok == "x" || tok == "y")
    }

    fn instruction_count(g: &Genome) -> usize {
        g.instructions.iter().filter(|i| i.op != OpCode::Const).count()
    }

    #[test]
    fn tree_size_distribution() {
        let mut rng = rand::thread_rng();
        let depth = crate::config::MAX_TREE_DEPTH;
        let n = 1000;

        let spatial_sizes: Vec<usize> = (0..n)
            .map(|_| {
                let g = Genome::new(Node::random_with_depth(&mut rng, depth));
                instruction_count(&g)
            })
            .collect();

        let palette_sizes: Vec<usize> = (0..n)
            .map(|_| {
                let g = Genome::new(Node::random_palette_with_depth(&mut rng, depth));
                instruction_count(&g)
            })
            .collect();

        let spatial_tiny  = spatial_sizes.iter().filter(|&&s| s <= 3).count();
        let spatial_small = spatial_sizes.iter().filter(|&&s| s > 3 && s <= 20).count();
        let spatial_large = spatial_sizes.iter().filter(|&&s| s > 20).count();
        let spatial_avg   = spatial_sizes.iter().sum::<usize>() as f64 / n as f64;

        let palette_tiny  = palette_sizes.iter().filter(|&&s| s <= 3).count();
        let palette_small = palette_sizes.iter().filter(|&&s| s > 3 && s <= 20).count();
        let palette_large = palette_sizes.iter().filter(|&&s| s > 20).count();
        let palette_avg   = palette_sizes.iter().sum::<usize>() as f64 / n as f64;

        let spatial_max = spatial_sizes.iter().copied().max().unwrap_or(0);
        let spatial_capped = spatial_sizes.iter().filter(|&&s| s >= MAX_INSTRUCTIONS - 1).count();
        let palette_max = palette_sizes.iter().copied().max().unwrap_or(0);
        let palette_capped = palette_sizes.iter().filter(|&&s| s >= MAX_INSTRUCTIONS - 1).count();

        println!("--- spatial (n={n}) ---");
        println!("  tiny  (≤3 instrs): {spatial_tiny} ({:.1}%)", spatial_tiny as f64 / n as f64 * 100.0);
        println!("  small (4-20):      {spatial_small} ({:.1}%)", spatial_small as f64 / n as f64 * 100.0);
        println!("  large (>20):       {spatial_large} ({:.1}%)", spatial_large as f64 / n as f64 * 100.0);
        println!("  avg instructions:  {spatial_avg:.1}");
        println!("  max instructions:  {spatial_max}");
        println!("  hitting cap (≥{}): {spatial_capped}", MAX_INSTRUCTIONS - 1);

        println!("--- palette (n={n}) ---");
        println!("  tiny  (≤3 instrs): {palette_tiny} ({:.1}%)", palette_tiny as f64 / n as f64 * 100.0);
        println!("  small (4-20):      {palette_small} ({:.1}%)", palette_small as f64 / n as f64 * 100.0);
        println!("  large (>20):       {palette_large} ({:.1}%)", palette_large as f64 / n as f64 * 100.0);
        println!("  avg instructions:  {palette_avg:.1}");
        println!("  max instructions:  {palette_max}");
        println!("  hitting cap (≥{}): {palette_capped}", MAX_INSTRUCTIONS - 1);
    }

    #[test]
    fn spatial_and_palette_genomes_are_independent() {
        let mut rng = rand::thread_rng();
        let depth = crate::config::MAX_TREE_DEPTH;

        for i in 0..8 {
            let h  = Genome::new(Node::random_with_depth(&mut rng, depth));
            let s  = Genome::new(Node::random_with_depth(&mut rng, depth));
            let v  = Genome::new(Node::random_with_depth(&mut rng, depth));
            let hr = Genome::new(Node::random_palette_with_depth(&mut rng, depth));
            let sr = Genome::new(Node::random_palette_with_depth(&mut rng, depth));
            let vr = Genome::new(Node::random_palette_with_depth(&mut rng, depth));

            let h_s  = h.to_expr_string();
            let s_s  = s.to_expr_string();
            let v_s  = v.to_expr_string();
            let hr_s = hr.to_expr_string();
            let sr_s = sr.to_expr_string();
            let vr_s = vr.to_expr_string();

            println!("--- individual {i} ---");
            println!("H:       {h_s}");
            println!("S:       {s_s}");
            println!("V:       {v_s}");
            println!("H_remap: {hr_s}");
            println!("S_remap: {sr_s}");
            println!("V_remap: {vr_s}");

            // Spatial genomes must reference x or y
            assert!(spatial_terminals(&h_s),  "H has no spatial terminal: {h_s}");
            assert!(spatial_terminals(&s_s),  "S has no spatial terminal: {s_s}");
            assert!(spatial_terminals(&v_s),  "V has no spatial terminal: {v_s}");

            // Remap genomes must NOT contain bare x or y
            assert!(no_spatial_terminals(&hr_s), "H_remap contains spatial terminal: {hr_s}");
            assert!(no_spatial_terminals(&sr_s), "S_remap contains spatial terminal: {sr_s}");
            assert!(no_spatial_terminals(&vr_s), "V_remap contains spatial terminal: {vr_s}");

            // Remap genomes must contain t
            assert!(has_palette_t(&hr_s), "H_remap has no t: {hr_s}");
            assert!(has_palette_t(&sr_s), "S_remap has no t: {sr_s}");
            assert!(has_palette_t(&vr_s), "V_remap has no t: {vr_s}");

            // Spatial and remap sets must not be identical as groups
            assert_ne!(h_s, hr_s, "individual {i}: H == H_remap");
            assert_ne!(s_s, sr_s, "individual {i}: S == S_remap");
            assert_ne!(v_s, vr_s, "individual {i}: V == V_remap");
        }
    }
}
