pub mod node;
pub mod linear;
pub mod op;

pub use node::Node;
pub use linear::{Genome, Instruction, OpCode};
pub use op::{op_def, Arity, OpDef, EvalFn};
