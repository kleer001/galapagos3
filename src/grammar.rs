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

const MAX_DEPTH: usize = 6;

pub fn build_expr(reader: &mut Reader, depth: usize) -> crate::expr::Expr {
    if depth > MAX_DEPTH {
        return terminal(reader);
    }

    match reader.next() % 6 {
        0 => terminal(reader),
        1 => crate::expr::Expr::Sin(Box::new(build_expr(reader, depth + 1))),
        2 => crate::expr::Expr::Cos(Box::new(build_expr(reader, depth + 1))),
        3 => crate::expr::Expr::Add(
            Box::new(build_expr(reader, depth + 1)),
            Box::new(build_expr(reader, depth + 1)),
        ),
        4 => crate::expr::Expr::Mul(
            Box::new(build_expr(reader, depth + 1)),
            Box::new(build_expr(reader, depth + 1)),
        ),
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
