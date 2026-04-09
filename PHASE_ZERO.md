## Phase 0 MVP — Grammar-Driven Image Generator (No Evolution Yet)

**Goal:**
A *deterministic*, minimal system:

* genome = random numbers
* grammar = maps numbers → expression
* expression = function f(x,y)
* renderer = CPU (no GPU yet)
* output = single image

No UI. No selection. No mutation.

---

# 1. Core Idea

```text
GENOME (random numbers)
        ↓
GRAMMAR (interprets numbers)
        ↓
EXPRESSION TREE
        ↓
FUNCTION f(x,y)
        ↓
IMAGE
```

---

# 2. Project Structure (Minimal)

```bash
src/
├── main.rs
├── genome.rs
├── grammar.rs
├── expr.rs
└── render.rs
```

---

# 3. Genome (Just Numbers)

```rust
// genome.rs
pub struct Genome {
    pub genes: Vec<u32>,
}

impl Genome {
    pub fn random(len: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            genes: (0..len).map(|_| rng.gen()).collect(),
        }
    }
}
```

---

# 4. Expression Tree

```rust
// expr.rs
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
```

---

# 5. Grammar (KEY PIECE)

Genome drives tree construction.

### Cursor-based gene reader

```rust
// grammar.rs
use crate::genome::Genome;

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
```

---

### Build Expression

```rust
use crate::expr::Expr;

const MAX_DEPTH: usize = 6;

pub fn build_expr(reader: &mut Reader, depth: usize) -> Expr {
    if depth > MAX_DEPTH {
        return terminal(reader);
    }

    match reader.next() % 6 {
        0 => terminal(reader),
        1 => Expr::Sin(Box::new(build_expr(reader, depth + 1))),
        2 => Expr::Cos(Box::new(build_expr(reader, depth + 1))),
        3 => Expr::Add(
            Box::new(build_expr(reader, depth + 1)),
            Box::new(build_expr(reader, depth + 1)),
        ),
        4 => Expr::Mul(
            Box::new(build_expr(reader, depth + 1)),
            Box::new(build_expr(reader, depth + 1)),
        ),
        _ => terminal(reader),
    }
}

fn terminal(reader: &mut Reader) -> Expr {
    match reader.next() % 3 {
        0 => Expr::X,
        1 => Expr::Y,
        _ => {
            let v = (reader.next() as f32 / u32::MAX as f32) * 2.0 - 1.0;
            Expr::Const(v)
        }
    }
}
```

---

# 6. Evaluator

```rust
// expr.rs
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
```

---

# 7. Renderer (CPU, simple)

```rust
// render.rs
use image::{ImageBuffer, Rgb};
use crate::expr::Expr;

pub fn render(expr: &Expr, width: u32, height: u32) {
    let mut img = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let nx = x as f32 / width as f32 * 2.0 - 1.0;
            let ny = y as f32 / height as f32 * 2.0 - 1.0;

            let v = expr.eval(nx, ny);

            let c = ((v * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;

            img.put_pixel(x, y, Rgb([c, c, c]));
        }
    }

    img.save("output.png").unwrap();
}
```

---

# 8. Main

```rust
// main.rs
mod genome;
mod grammar;
mod expr;
mod render;

use genome::Genome;
use grammar::{Reader, build_expr};
use render::render;

fn main() {
    let genome = Genome::random(64);

    let mut reader = Reader::new(&genome.genes);
    let expr = build_expr(&mut reader, 0);

    render(&expr, 1024, 1024);

    println!("Rendered to output.png");
}
```

---

# 9. Cargo Dependency

```toml
[dependencies]
rand = "0.8"
image = "0.24"
```

---

# 10. What This Gives You

* Deterministic mapping: genome → image
* Infinite variation from random seeds
* Grammar-based structure (Sims-style foundation)
* No GPU complexity yet
* Easy to debug

---

# 11. Validation Checklist

* [ ] runs instantly
* [ ] outputs PNG
* [ ] different seeds → different images
* [ ] no crashes
* [ ] expressions stay bounded

