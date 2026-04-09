pub struct Reader<'a> {
    genes: &'a [u32],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(genes: &'a [u32]) -> Self {
        Self { genes, pos: 0 }
    }

    pub fn next(&mut self) -> u32 {
        let val = self.genes[self.pos % self.genes.len()];
        self.pos += 1;
        val
    }
}

const MAX_DEPTH: usize = 12;
const TERMINAL_CHANCE: u32 = 1; // 1% chance to hit terminal

pub fn build_expr(reader: &mut Reader, depth: usize) -> crate::expr::Expr {
    if depth > MAX_DEPTH {
        return terminal(reader);
    }

    // Roll for terminal (1% chance)
    if reader.next() % 100 < TERMINAL_CHANCE {
        return terminal(reader);
    }

    match reader.next() % 15 {
        0 => crate::expr::Expr::Add(
            Box::new(build_expr(reader, depth + 1)),
            Box::new(build_expr(reader, depth + 1)),
        ),
        1 => crate::expr::Expr::Sub(
            Box::new(build_expr(reader, depth + 1)),
            Box::new(build_expr(reader, depth + 1)),
        ),
        2 => crate::expr::Expr::Mul(
            Box::new(build_expr(reader, depth + 1)),
            Box::new(build_expr(reader, depth + 1)),
        ),
        3 => crate::expr::Expr::Div(
            Box::new(build_expr(reader, depth + 1)),
            Box::new(build_expr(reader, depth + 1)),
        ),
        4 => crate::expr::Expr::Sin(Box::new(build_expr(reader, depth + 1))),
        5 => crate::expr::Expr::Cos(Box::new(build_expr(reader, depth + 1))),
        6 => crate::expr::Expr::Tan(Box::new(build_expr(reader, depth + 1))),
        7 => crate::expr::Expr::Abs(Box::new(build_expr(reader, depth + 1))),
        8 => crate::expr::Expr::Sqrt(Box::new(build_expr(reader, depth + 1))),
        9 => crate::expr::Expr::Pow(
            Box::new(build_expr(reader, depth + 1)),
            Box::new(build_expr(reader, depth + 1)),
        ),
        10 => crate::expr::Expr::Exp(Box::new(build_expr(reader, depth + 1))),
        _ => terminal(reader),
    }
}

fn terminal(reader: &mut Reader) -> crate::expr::Expr {
    match reader.next() % 3 {
        0 => crate::expr::Expr::X,
        1 => crate::expr::Expr::Y,
        _ => {
            let v = (reader.next() as f32 / u32::MAX as f32) * 2.0 - 1.0;
            crate::expr::Expr::Const(v)
        }
    }
}
