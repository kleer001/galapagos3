#[derive(Clone)]
pub enum Expr {
    X,
    Y,
    Const(f32),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Sin(Box<Expr>),
    Cos(Box<Expr>),
    Tan(Box<Expr>),
    Abs(Box<Expr>),
    Sqrt(Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Exp(Box<Expr>),
}

impl Expr {
    pub fn eval(&self, x: f32, y: f32) -> f32 {
        match self {
            Expr::X => x,
            Expr::Y => y,
            Expr::Const(c) => *c,
            Expr::Add(a, b) => a.eval(x, y) + b.eval(x, y),
            Expr::Sub(a, b) => a.eval(x, y) - b.eval(x, y),
            Expr::Mul(a, b) => a.eval(x, y) * b.eval(x, y),
            Expr::Div(a, b) => a.eval(x, y) / (b.eval(x, y).max(0.001)),
            Expr::Sin(a) => a.eval(x, y).sin(),
            Expr::Cos(a) => a.eval(x, y).cos(),
            Expr::Tan(a) => a.eval(x, y).tan(),
            Expr::Abs(a) => a.eval(x, y).abs(),
            Expr::Sqrt(a) => a.eval(x, y).sqrt().max(0.0),
            Expr::Pow(a, b) => a.eval(x, y).powf(b.eval(x, y)),
            Expr::Exp(a) => a.eval(x, y).exp(),
        }
    }
}
