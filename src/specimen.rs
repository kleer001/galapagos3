//! Reloadable on-disk genome format.
//!
//! The interactive app's "save" path writes a PNG plus a human-readable
//! expression dump, neither of which can reconstruct a genome. A `Specimen`
//! is the missing round-trippable unit: the six channel genomes that define
//! one individual (h, s, v, h_remap, s_remap, v_remap) plus its color model.
//! The breeder writes these; the animated widget loads them.

use std::fs;
use std::io;
use std::path::Path;

use crate::genome::{opcode_from_u32, Genome, Instruction};

/// Channel genomes per individual, in upload order.
pub const CHANNEL_COUNT: usize = 6;

pub struct Specimen {
    /// h, s, v, h_remap, s_remap, v_remap — the order the renderer expects.
    pub channels: [Genome; CHANNEL_COUNT],
    pub color_model: u32,
}

const MAGIC: &str = "galapagos-specimen";
const VERSION: u32 = 1;

/// Serialize a specimen to a compact line-oriented text file.
pub fn save(path: &Path, spec: &Specimen) -> io::Result<()> {
    let mut out = String::new();
    out.push_str(&format!("{MAGIC} {VERSION}\n"));
    out.push_str(&format!("color_model {}\n", spec.color_model));
    for (ci, g) in spec.channels.iter().enumerate() {
        out.push_str(&format!("channel {} {}\n", ci, g.instructions.len()));
        for i in &g.instructions {
            out.push_str(&format!(
                "{} {} {} {} {}\n",
                i.op as u32, i.a, i.b, i.c, i.value
            ));
        }
    }
    fs::write(path, out)
}

/// Parse a specimen written by [`save`]. Errors are descriptive strings so the
/// widget can skip a bad file and keep cycling the rest of the library.
pub fn load(path: &Path) -> Result<Specimen, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut lines = text.lines();

    let mut header = lines.next().ok_or("empty file")?.split_whitespace();
    if header.next() != Some(MAGIC) {
        return Err(format!("not a specimen file: {}", path.display()));
    }
    let ver: u32 = header
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or("missing version")?;
    if ver != VERSION {
        return Err(format!("unsupported specimen version {ver}"));
    }

    let mut cm = lines
        .next()
        .ok_or("missing color_model")?
        .split_whitespace();
    if cm.next() != Some("color_model") {
        return Err("expected color_model line".to_string());
    }
    let color_model: u32 = cm
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or("bad color_model")?;

    let mut channels: Vec<Genome> = Vec::with_capacity(CHANNEL_COUNT);
    for ci in 0..CHANNEL_COUNT {
        let mut head = lines
            .next()
            .ok_or(format!("missing channel {ci}"))?
            .split_whitespace();
        if head.next() != Some("channel") {
            return Err(format!("expected channel header {ci}"));
        }
        let _idx: usize = head
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or("bad channel index")?;
        let count: usize = head
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or("bad channel count")?;

        let mut instructions = Vec::with_capacity(count);
        for _ in 0..count {
            let line = lines.next().ok_or("truncated instruction list")?;
            let mut p = line.split_whitespace();
            let op_n: u32 = p.next().and_then(|s| s.parse().ok()).ok_or("bad op")?;
            let a: i32 = p
                .next()
                .and_then(|s| s.parse().ok())
                .ok_or("bad operand a")?;
            let b: i32 = p
                .next()
                .and_then(|s| s.parse().ok())
                .ok_or("bad operand b")?;
            let c: i32 = p
                .next()
                .and_then(|s| s.parse().ok())
                .ok_or("bad operand c")?;
            let value: f32 = p.next().and_then(|s| s.parse().ok()).ok_or("bad value")?;
            let op = opcode_from_u32(op_n).ok_or(format!("unknown opcode {op_n}"))?;
            instructions.push(Instruction { op, a, b, c, value });
        }
        channels.push(Genome { instructions });
    }

    let channels: [Genome; CHANNEL_COUNT] = channels
        .try_into()
        .map_err(|_| "channel count mismatch".to_string())?;
    Ok(Specimen {
        channels,
        color_model,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::OpCode;

    #[test]
    fn round_trip_preserves_genomes() {
        let mk = |seed: f32| Genome {
            instructions: vec![
                Instruction { op: OpCode::X, a: -1, b: -1, c: -1, value: 0.0 },
                Instruction { op: OpCode::Const, a: -1, b: -1, c: -1, value: seed },
                Instruction { op: OpCode::Add, a: 0, b: 1, c: -1, value: 0.0 },
            ],
        };
        let spec = Specimen {
            channels: std::array::from_fn(|i| mk(i as f32 * 0.25)),
            color_model: 3,
        };

        let path = std::env::temp_dir().join(format!("galapagos_roundtrip_{}.gal", std::process::id()));
        save(&path, &spec).unwrap();
        let loaded = load(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded.color_model, spec.color_model);
        for i in 0..CHANNEL_COUNT {
            assert_eq!(spec.channels[i].to_raw(), loaded.channels[i].to_raw());
        }
    }

    #[test]
    fn rejects_foreign_files() {
        let path = std::env::temp_dir().join(format!("galapagos_bad_{}.gal", std::process::id()));
        std::fs::write(&path, "not a genome\n").unwrap();
        let result = load(&path);
        std::fs::remove_file(&path).ok();
        assert!(result.is_err());
    }
}
