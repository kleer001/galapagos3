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
    pub fn random(width: u32, height: u32, image_idx: usize) -> Self {
        let genes: Vec<u32> = (0..64).map(|_| rand::random::<u32>()).collect();
        Expr::random_seed(width, height, image_idx)
    }

    pub fn random_seed(width: u32, height: u32, image_idx: usize) -> Self {
        // Create a seeded pseudo-random generator based on dimensions and image index
        let seed = ((width as u64 * 31337) ^ (height as u64 * 65689) ^ (image_idx as u64 * 12345)) as u32;
        let mut state = seed % u32::MAX;

        fn next_val(state: &mut u32) -> u32 {
            *state = (*state as u64 * 1664525 + 1013904223) as u32;
            *state
        }

        // Build a more complex expression tree using the grammar approach
        let genes: Vec<u32> = (0..128).map(|_| next_val(&mut state)).collect();
        use crate::grammar::{Reader, build_expr};
        let mut reader = Reader::new(&genes);
        build_expr(&mut reader, 0)
    }

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
