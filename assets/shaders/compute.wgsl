// Galápagos 3 - WGSL Bytecode Interpreter Compute Shader
// Evaluates expression trees encoded as stack-machine bytecode

struct Instruction {
    op: u32,
    a: i32,
    b: i32,
    c: i32,
    value: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

// Match Rust OpCode enum exactly
const OP_X: u32 = 0;
const OP_Y: u32 = 1;
const OP_CONST: u32 = 2;
const OP_SIN: u32 = 3;
const OP_COS: u32 = 4;
const OP_TAN: u32 = 5;
const OP_ABS: u32 = 6;
const OP_SQRT: u32 = 7;
const OP_LOG: u32 = 8;
const OP_EXP: u32 = 9;
const OP_FRACT: u32 = 10;
const OP_ADD: u32 = 11;
const OP_SUB: u32 = 12;
const OP_MUL: u32 = 13;
const OP_DIV: u32 = 14;
const OP_POW: u32 = 15;
const OP_MIX: u32 = 16;
const OP_SMOOTHSTEP: u32 = 17;
const OP_LENGTH: u32 = 18;
const OP_DOT: u32 = 19;
// Phase 2 operators
const OP_ACOS: u32 = 20;
const OP_ASIN: u32 = 21;
const OP_ATAN: u32 = 22;
const OP_SINH: u32 = 23;
const OP_COSH: u32 = 24;
const OP_TANH: u32 = 25;
const OP_MIN: u32 = 26;
const OP_MAX: u32 = 27;
const OP_CLAMP: u32 = 28;
const OP_SIGN: u32 = 29;
const OP_FLOOR: u32 = 30;
const OP_CEIL: u32 = 31;
const OP_ROUND: u32 = 32;
const OP_NEGATE: u32 = 33;
const OP_STEP: u32 = 34;
const OP_RECIPROCAL: u32 = 35;
const OP_INVERT: u32 = 36;
const OP_RADIAL: u32 = 37;

// Maximum stack depth for interpreter
const MAX_STACK: u32 = 64;
// Instructions per genome
const INSTRUCTIONS_PER_GENOME: u32 = 64;

struct OutputInfo {
    width: u32,
    height: u32,
};

// Flat storage buffer of all instructions
@group(0) @binding(0)
var<storage> all_instructions: array<Instruction>;

@group(0) @binding(1)
var<uniform> output_info: OutputInfo;

@group(0) @binding(2)
var<storage, read_write> output: array<vec4<f32>>;

// HSV to RGB conversion
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    if (s == 0.0) {
        let c = v;
        return vec3<f32>(c, c, c);
    }
    let i = u32(floor(h * 6.0)) % 6u;
    let f = h * 6.0 - f32(i);
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    var rgb: vec3<f32>;
    switch (i) {
        case 0u { rgb = vec3<f32>(v, t, p); }
        case 1u { rgb = vec3<f32>(q, v, p); }
        case 2u { rgb = vec3<f32>(p, v, t); }
        case 3u { rgb = vec3<f32>(p, q, v); }
        case 4u { rgb = vec3<f32>(t, p, v); }
        case 5u { rgb = vec3<f32>(v, p, q); }
        default { rgb = vec3<f32>(0.0, 0.0, 0.0); }
    }
    return rgb;
}

// Bytecode interpreter - evaluates a genome at given coordinates
fn evaluate(base_idx: u32, nx: f32, ny: f32) -> f32 {
    var stack: array<f32, 64>;
    var sp: u32 = 0u;

    // Find the last non-Const instruction
    var real_end: u32 = 0u;
    for (var i: u32 = 0u; i < INSTRUCTIONS_PER_GENOME; i += 1u) {
        if (all_instructions[base_idx + i].op != OP_CONST) {
            real_end = i;
        }
    }

    // Execute bytecode up to real_end
    for (var i: u32 = 0u; i <= real_end; i += 1u) {
        let instr = all_instructions[base_idx + i];
        var result: f32 = 0.0;

        switch (instr.op) {
            case OP_X { result = nx; }
            case OP_Y { result = ny; }
            case OP_CONST { result = instr.value; }
            case OP_SIN {
                let idx = u32(instr.a);
                if (idx < sp) { result = sin(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_COS {
                let idx = u32(instr.a);
                if (idx < sp) { result = cos(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_TAN {
                let idx = u32(instr.a);
                if (idx < sp) { result = tan(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_ABS {
                let idx = u32(instr.a);
                if (idx < sp) { result = abs(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_SQRT {
                let idx = u32(instr.a);
                if (idx < sp) { result = sqrt(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_LOG {
                let idx = u32(instr.a);
                if (idx < sp) { result = log(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_EXP {
                let idx = u32(instr.a);
                if (idx < sp) { result = exp(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_FRACT {
                let idx = u32(instr.a);
                if (idx < sp) { result = fract(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_ADD {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) { result = stack[a_idx] + stack[b_idx]; }
                else { result = 0.0; }
            }
            case OP_SUB {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) { result = stack[a_idx] - stack[b_idx]; }
                else { result = 0.0; }
            }
            case OP_MUL {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) { result = stack[a_idx] * stack[b_idx]; }
                else { result = 0.0; }
            }
            case OP_DIV {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) {
                    if (abs(stack[b_idx]) > 1e-6) { result = stack[a_idx] / stack[b_idx]; }
                    else { result = 0.0; }
                } else { result = 0.0; }
            }
            case OP_POW {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) { result = pow(stack[a_idx], stack[b_idx]); }
                else { result = 0.0; }
            }
            case OP_MIX {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                let c_idx = u32(instr.c);
                if (a_idx < sp && b_idx < sp && c_idx < sp) { result = mix(stack[a_idx], stack[b_idx], stack[c_idx]); }
                else { result = 0.0; }
            }
            case OP_SMOOTHSTEP {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                let c_idx = u32(instr.c);
                if (a_idx < sp && b_idx < sp && c_idx < sp) {
                    let t = clamp((stack[c_idx] - stack[a_idx]) / (stack[b_idx] - stack[a_idx]), 0.0, 1.0);
                    result = t * t * (3.0 - 2.0 * t);
                } else { result = 0.0; }
            }
            case OP_LENGTH {
                let idx = u32(instr.a);
                if (idx < sp) { result = abs(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_DOT {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) { result = stack[a_idx] * stack[b_idx]; }
                else { result = 0.0; }
            }
            // Phase 2 operators
            case OP_ACOS {
                let idx = u32(instr.a);
                if (idx < sp) { result = acos(clamp(stack[idx], -1.0, 1.0)); }
                else { result = 0.0; }
            }
            case OP_ASIN {
                let idx = u32(instr.a);
                if (idx < sp) { result = asin(clamp(stack[idx], -1.0, 1.0)); }
                else { result = 0.0; }
            }
            case OP_ATAN {
                let idx = u32(instr.a);
                if (idx < sp) { result = atan(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_SINH {
                let idx = u32(instr.a);
                if (idx < sp) { result = sinh(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_COSH {
                let idx = u32(instr.a);
                if (idx < sp) { result = cosh(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_TANH {
                let idx = u32(instr.a);
                if (idx < sp) { result = tanh(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_MIN {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) { result = min(stack[a_idx], stack[b_idx]); }
                else { result = 0.0; }
            }
            case OP_MAX {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) { result = max(stack[a_idx], stack[b_idx]); }
                else { result = 0.0; }
            }
            case OP_CLAMP {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                let c_idx = u32(instr.c);
                if (a_idx < sp && b_idx < sp && c_idx < sp) { result = clamp(stack[a_idx], stack[b_idx], stack[c_idx]); }
                else { result = 0.0; }
            }
            case OP_SIGN {
                let idx = u32(instr.a);
                if (idx < sp) { result = sign(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_FLOOR {
                let idx = u32(instr.a);
                if (idx < sp) { result = floor(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_CEIL {
                let idx = u32(instr.a);
                if (idx < sp) { result = ceil(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_ROUND {
                let idx = u32(instr.a);
                if (idx < sp) { result = round(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_NEGATE {
                let idx = u32(instr.a);
                if (idx < sp) { result = -stack[idx]; }
                else { result = 0.0; }
            }
            case OP_STEP {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) { result = select(0.0, 1.0, stack[b_idx] >= stack[a_idx]); }
                else { result = 0.0; }
            }
            case OP_RECIPROCAL {
                let idx = u32(instr.a);
                if (idx < sp) {
                    if (abs(stack[idx]) > 1e-6) { result = 1.0 / stack[idx]; }
                    else { result = 0.0; }
                } else { result = 0.0; }
            }
            case OP_INVERT {
                let idx = u32(instr.a);
                if (idx < sp) { result = 1.0 - stack[idx]; }
                else { result = 0.0; }
            }
            case OP_RADIAL {
                result = sqrt(nx * nx + ny * ny);
            }
            default { result = 0.0; }
        }

        if (sp < MAX_STACK) {
            stack[sp] = result;
            sp += 1u;
        }
    }

    // Return top of stack
    var raw: f32 = 0.0;
    if (sp > 0u) { raw = stack[sp - 1u]; }
    return fract(fract(raw) + 1.0);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tile_w = 256u;
    let tile_h = 256u;
    let grid_cols = 4u;

    // Check if we're in bounds
    if (global_id.x >= output_info.width || global_id.y >= output_info.height) {
        return;
    }

    // Calculate which tile and position within tile
    let tile_x = global_id.x / tile_w;
    let tile_y = global_id.y / tile_h;
    let local_x = global_id.x % tile_w;
    let local_y = global_id.y % tile_h;

    // Tile index in grid
    let tile_idx = tile_y * grid_cols + tile_x;

    // Individual index (0-15)
    let ind_idx = tile_idx;

    // Genome indices for H, S, V channels
    let h_genome_idx = ind_idx * 3u;
    let s_genome_idx = ind_idx * 3u + 1u;
    let v_genome_idx = ind_idx * 3u + 2u;

    // Normalize coordinates to [-1, 1]
    let nx = f32(local_x) / f32(tile_w) * 2.0 - 1.0;
    let ny = f32(local_y) / f32(tile_h) * 2.0 - 1.0;

    // Evaluate H, S, V channels using the interpreter
    let h = evaluate(h_genome_idx * INSTRUCTIONS_PER_GENOME, nx, ny);
    let s = evaluate(s_genome_idx * INSTRUCTIONS_PER_GENOME, nx, ny);
    let v = evaluate(v_genome_idx * INSTRUCTIONS_PER_GENOME, nx, ny);

    // Convert to RGB and output
    let rgb = hsv_to_rgb(h, s, v);
    let out_idx = global_id.y * output_info.width + global_id.x;
    output[out_idx] = vec4<f32>(rgb, 1.0);
}
