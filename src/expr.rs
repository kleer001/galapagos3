#[derive(Clone)]
pub enum Expr {
    X,
    Y,
    Const(f32),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Sin(Box<Expr>),
    Cos(Box<Expr>),
}

impl Expr {
    pub fn eval(&self, x: f32, y: f32) -> f32 {
        match self {
            Expr::X => x,
            Expr::Y => y,
            Expr::Const(c) => *c,
            Expr::Add(a, b) => a.eval(x, y) + b.eval(x, y),
            Expr::Mul(a, b) => a.eval(x, y) * b.eval(x, y),
            Expr::Sin(a) => a.eval(x, y).sin(),
            Expr::Cos(a) => a.eval(x, y).cos(),
        }
    }
}
