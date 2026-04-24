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
// Phase 2 operators
const OP_ACOS: u32 = 18;
const OP_ASIN: u32 = 19;
const OP_ATAN: u32 = 20;
const OP_SINH: u32 = 21;
const OP_COSH: u32 = 22;
const OP_TANH: u32 = 23;
const OP_MIN: u32 = 24;
const OP_MAX: u32 = 25;
const OP_CLAMP: u32 = 26;
const OP_SIGN: u32 = 27;
const OP_FLOOR: u32 = 28;
const OP_NEGATE: u32 = 29;
const OP_STEP: u32 = 30;
const OP_RECIPROCAL: u32 = 31;
const OP_INVERT: u32 = 32;
// Phase 3 operators
const OP_VALUE_NOISE: u32 = 33;
const OP_FBM: u32 = 34;
const OP_MIRROR_X: u32 = 35;
const OP_MIRROR_Y: u32 = 36;
// New operators
const OP_ATAN2: u32 = 37;
const OP_MOD: u32 = 38;
const OP_WORLEY: u32 = 39;
const OP_TRIWAVE: u32 = 40;
const OP_CHEBYSHEV: u32 = 41;
const OP_MANHATTAN: u32 = 42;
const OP_SINFOLD: u32 = 43;
const OP_PALETTE_T: u32 = 44;
// Noise variants
const OP_TURBULENCE: u32 = 45;
const OP_RIDGED: u32 = 46;
const OP_BILLOW: u32 = 47;
const OP_SIMPLEX_NOISE: u32 = 48;
const OP_DOMAIN_WARP: u32 = 49;
const OP_SCALED_X: u32 = 50;
const OP_SCALED_Y: u32 = 51;

// Maximum stack depth for interpreter (auto-generated from config.rs)
const MAX_STACK: u32 = 256;
// Instructions per genome (auto-generated from config.rs)
const INSTRUCTIONS_PER_GENOME: u32 = 256;

struct OutputInfo {
    width: u32,
    height: u32,
    tile_w: u32,
    tile_h: u32,
    jitter_x: f32,
    jitter_y: f32,
    // 0=HSV, 1=RGB, 2=HSL, 3=CMY, 4=YUV (BT.601). Chosen per-render by the host.
    color_model: u32,
    _pad: u32,
};

// Flat storage buffer of all instructions (6 genomes: H S V H_remap S_remap V_remap)
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

// One of three HSL hue→channel lookups; `t` is the hue offset for this channel.
fn hsl_hue_to_c(p: f32, q: f32, t_in: f32) -> f32 {
    var t = fract(fract(t_in) + 1.0);
    if (t < 1.0 / 6.0) { return p + (q - p) * 6.0 * t; }
    if (t < 0.5) { return q; }
    if (t < 2.0 / 3.0) { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    return p;
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> vec3<f32> {
    if (s == 0.0) { return vec3<f32>(l, l, l); }
    var q: f32;
    if (l < 0.5) { q = l * (1.0 + s); } else { q = l + s - l * s; }
    let p = 2.0 * l - q;
    return vec3<f32>(
        hsl_hue_to_c(p, q, h + 1.0 / 3.0),
        hsl_hue_to_c(p, q, h),
        hsl_hue_to_c(p, q, h - 1.0 / 3.0),
    );
}

// BT.601: U and V arrive in [0,1] and are recentered to [-0.5, 0.5].
fn yuv_to_rgb(y: f32, u_in: f32, v_in: f32) -> vec3<f32> {
    let u = u_in - 0.5;
    let v = v_in - 0.5;
    return vec3<f32>(
        y + 1.402 * v,
        y - 0.344136 * u - 0.714136 * v,
        y + 1.772 * u,
    );
}

// Dispatch three channels (each in [0,1]) to linear RGB according to model id.
fn channels_to_rgb(model: u32, c0: f32, c1: f32, c2: f32) -> vec3<f32> {
    switch (model) {
        case 0u { return hsv_to_rgb(c0, c1, c2); }
        case 1u { return vec3<f32>(c0, c1, c2); }
        case 2u { return hsl_to_rgb(c0, c1, c2); }
        case 3u { return vec3<f32>(1.0 - c0, 1.0 - c1, 1.0 - c2); }
        case 4u { return yuv_to_rgb(c0, c1, c2); }
        default { return hsv_to_rgb(c0, c1, c2); }
    }
}

// Bytecode interpreter - evaluates a genome at given coordinates.
// t = raw channel value; used only by palette remap genomes (OP_PALETTE_T).
fn evaluate(base_idx: u32, nx: f32, ny: f32, t: f32) -> f32 {
    var stack: array<f32, MAX_STACK>;
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
                if (idx < sp) { let t = tan(stack[idx]); result = select(0.0, t, t == t && abs(t) < 1e10); }
                else { result = 0.0; }
            }
            case OP_ABS {
                let idx = u32(instr.a);
                if (idx < sp) { result = abs(stack[idx]); }
                else { result = 0.0; }
            }
            case OP_SQRT {
                let idx = u32(instr.a);
                if (idx < sp) { result = sqrt(max(stack[idx], 0.0)); }
                else { result = 0.0; }
            }
            case OP_LOG {
                let idx = u32(instr.a);
                if (idx < sp) { result = select(0.0, log(stack[idx]), stack[idx] > 0.0); }
                else { result = 0.0; }
            }
            case OP_EXP {
                let idx = u32(instr.a);
                if (idx < sp) { let e = exp(stack[idx]); result = select(0.0, e, e == e && e < 1e38); }
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
                if (a_idx < sp && b_idx < sp) {
                    if (stack[a_idx] > 0.0) { result = pow(stack[a_idx], stack[b_idx]); }
                    else { result = 0.0; }
                } else { result = 0.0; }
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
                    let denom = stack[b_idx] - stack[a_idx];
                    if (abs(denom) < 1e-6) {
                        result = 0.0;
                    } else {
                        let t = clamp((stack[c_idx] - stack[a_idx]) / denom, 0.0, 1.0);
                        result = t * t * (3.0 - 2.0 * t);
                    }
                } else { result = 0.0; }
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
                if (a_idx < sp && b_idx < sp && c_idx < sp) {
                    let lo = min(stack[b_idx], stack[c_idx]);
                    let hi = max(stack[b_idx], stack[c_idx]);
                    result = clamp(stack[a_idx], lo, hi);
                } else { result = 0.0; }
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
            // Phase 3 operators
            case OP_VALUE_NOISE {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) {
                    let vx = stack[a_idx];
                    let vy = stack[b_idx];
                    let xi = floor(vx);
                    let yi = floor(vy);
                    let fx = vx - xi;
                    let fy = vy - yi;
                    let fa = sin(xi * 127.1 + yi * 311.3);
                    let fb = sin((xi + 1.0) * 127.1 + yi * 311.3);
                    let fc = sin(xi * 127.1 + (yi + 1.0) * 311.3);
                    let fd = sin((xi + 1.0) * 127.1 + (yi + 1.0) * 311.3);
                    let top = mix(fa, fb, fx);
                    let bottom = mix(fc, fd, fx);
                    result = mix(top, bottom, fy);
                } else { result = 0.0; }
            }
            case OP_FBM {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                let octaves = i32(instr.c);
                if (a_idx < sp && b_idx < sp) {
                    var value: f32 = 0.0;
                    var amplitude: f32 = 1.0;
                    var frequency: f32 = 1.0;
                    var max_val: f32 = 0.0;
                    for (var o: i32 = 0; o < max(1, min(8, octaves)); o += 1) {
                        let vx = stack[a_idx] * frequency;
                        let vy = stack[b_idx] * frequency;
                        let xi = floor(vx);
                        let yi = floor(vy);
                        let fx = vx - xi;
                        let fy = vy - yi;
                        let fa = sin(xi * 127.1 + yi * 311.3);
                        let fb = sin((xi + 1.0) * 127.1 + yi * 311.3);
                        let fc = sin(xi * 127.1 + (yi + 1.0) * 311.3);
                        let fd = sin((xi + 1.0) * 127.1 + (yi + 1.0) * 311.3);
                        let top = mix(fa, fb, fx);
                        let bottom = mix(fc, fd, fx);
                        let noise = mix(top, bottom, fy);
                        value += noise * amplitude;
                        max_val += amplitude;
                        amplitude *= 0.5;
                        frequency *= 2.0;
                    }
                    if (max_val > 0.0) { result = value / max_val; } else { result = 0.0; }
                } else { result = 0.0; }
            }
            case OP_MIRROR_X {
                result = abs(nx);
            }
            case OP_MIRROR_Y {
                result = abs(ny);
            }
            // New operators
            case OP_ATAN2 {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) { result = atan2(stack[a_idx], stack[b_idx]); }
                else { result = 0.0; }
            }
            case OP_MOD {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) {
                    let bv = stack[b_idx];
                    if (abs(bv) > 1e-6) { result = stack[a_idx] - bv * floor(stack[a_idx] / bv); }
                    else { result = 0.0; }
                } else { result = 0.0; }
            }
            case OP_WORLEY {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) {
                    let vx = stack[a_idx];
                    let vy = stack[b_idx];
                    let xi = floor(vx);
                    let yi = floor(vy);
                    var min_dist: f32 = 1e9;
                    for (var dy: i32 = -1; dy <= 1; dy += 1) {
                        for (var dx: i32 = -1; dx <= 1; dx += 1) {
                            let cx = xi + f32(dx);
                            let cy = yi + f32(dy);
                            let h1 = (sin(cx * 127.1 + cy * 311.3) + 1.0) * 0.5;
                            let h2 = (sin(cx * 269.5 + cy * 183.3) + 1.0) * 0.5;
                            let px = cx + h1;
                            let py = cy + h2;
                            let d = sqrt((vx - px) * (vx - px) + (vy - py) * (vy - py));
                            if (d < min_dist) { min_dist = d; }
                        }
                    }
                    result = min(min_dist, 1.0);
                } else { result = 0.0; }
            }
            case OP_TRIWAVE {
                let idx = u32(instr.a);
                if (idx < sp) { result = abs(fract(stack[idx] * 0.5) * 2.0 - 1.0); }
                else { result = 0.0; }
            }
            case OP_CHEBYSHEV {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) { result = max(abs(stack[a_idx]), abs(stack[b_idx])); }
                else { result = 0.0; }
            }
            case OP_MANHATTAN {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) { result = abs(stack[a_idx]) + abs(stack[b_idx]); }
                else { result = 0.0; }
            }
            case OP_SINFOLD {
                let idx = u32(instr.a);
                if (idx < sp) { result = sin(stack[idx] * 3.14159265); }
                else { result = 0.0; }
            }
            case OP_PALETTE_T { result = t; }
            case OP_TURBULENCE {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                let octaves = i32(instr.c);
                if (a_idx < sp && b_idx < sp) {
                    var value: f32 = 0.0;
                    var amplitude: f32 = 1.0;
                    var frequency: f32 = 1.0;
                    var max_val: f32 = 0.0;
                    for (var o: i32 = 0; o < max(1, min(8, octaves)); o += 1) {
                        let vx = stack[a_idx] * frequency;
                        let vy = stack[b_idx] * frequency;
                        let xi = floor(vx); let yi = floor(vy);
                        let fx = vx - xi; let fy = vy - yi;
                        let fa = sin(xi * 127.1 + yi * 311.3);
                        let fb = sin((xi + 1.0) * 127.1 + yi * 311.3);
                        let fc = sin(xi * 127.1 + (yi + 1.0) * 311.3);
                        let fd = sin((xi + 1.0) * 127.1 + (yi + 1.0) * 311.3);
                        let noise = mix(mix(fa, fb, fx), mix(fc, fd, fx), fy);
                        value += abs(noise) * amplitude;
                        max_val += amplitude;
                        amplitude *= 0.5;
                        frequency *= 2.0;
                    }
                    if (max_val > 0.0) { result = value / max_val; } else { result = 0.0; }
                } else { result = 0.0; }
            }
            case OP_RIDGED {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                let octaves = i32(instr.c);
                if (a_idx < sp && b_idx < sp) {
                    var value: f32 = 0.0;
                    var amplitude: f32 = 1.0;
                    var frequency: f32 = 1.0;
                    var max_val: f32 = 0.0;
                    for (var o: i32 = 0; o < max(1, min(8, octaves)); o += 1) {
                        let vx = stack[a_idx] * frequency;
                        let vy = stack[b_idx] * frequency;
                        let xi = floor(vx); let yi = floor(vy);
                        let fx = vx - xi; let fy = vy - yi;
                        let fa = sin(xi * 127.1 + yi * 311.3);
                        let fb = sin((xi + 1.0) * 127.1 + yi * 311.3);
                        let fc = sin(xi * 127.1 + (yi + 1.0) * 311.3);
                        let fd = sin((xi + 1.0) * 127.1 + (yi + 1.0) * 311.3);
                        let noise = mix(mix(fa, fb, fx), mix(fc, fd, fx), fy);
                        value += (1.0 - abs(noise)) * amplitude;
                        max_val += amplitude;
                        amplitude *= 0.5;
                        frequency *= 2.0;
                    }
                    if (max_val > 0.0) { result = value / max_val; } else { result = 0.0; }
                } else { result = 0.0; }
            }
            case OP_BILLOW {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                let octaves = i32(instr.c);
                if (a_idx < sp && b_idx < sp) {
                    var value: f32 = 0.0;
                    var amplitude: f32 = 1.0;
                    var frequency: f32 = 1.0;
                    var max_val: f32 = 0.0;
                    for (var o: i32 = 0; o < max(1, min(8, octaves)); o += 1) {
                        let vx = stack[a_idx] * frequency;
                        let vy = stack[b_idx] * frequency;
                        let xi = floor(vx); let yi = floor(vy);
                        let fx = vx - xi; let fy = vy - yi;
                        let fa = sin(xi * 127.1 + yi * 311.3);
                        let fb = sin((xi + 1.0) * 127.1 + yi * 311.3);
                        let fc = sin(xi * 127.1 + (yi + 1.0) * 311.3);
                        let fd = sin((xi + 1.0) * 127.1 + (yi + 1.0) * 311.3);
                        let noise = mix(mix(fa, fb, fx), mix(fc, fd, fx), fy);
                        value += (abs(noise) * 2.0 - 1.0) * amplitude;
                        max_val += amplitude;
                        amplitude *= 0.5;
                        frequency *= 2.0;
                    }
                    if (max_val > 0.0) { result = value / max_val; } else { result = 0.0; }
                } else { result = 0.0; }
            }
            case OP_SIMPLEX_NOISE {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                if (a_idx < sp && b_idx < sp) {
                    let vx = stack[a_idx];
                    let vy = stack[b_idx];
                    let F2 = 0.366025404;
                    let G2 = 0.211324865;
                    let s = (vx + vy) * F2;
                    let i = floor(vx + s);
                    let j = floor(vy + s);
                    let t0 = (i + j) * G2;
                    let x0 = vx - i + t0;
                    let y0 = vy - j + t0;
                    var i1: f32; var j1: f32;
                    if (x0 > y0) { i1 = 1.0; j1 = 0.0; } else { i1 = 0.0; j1 = 1.0; }
                    let x1 = x0 - i1 + G2;
                    let y1 = y0 - j1 + G2;
                    let x2 = x0 - 1.0 + 2.0 * G2;
                    let y2 = y0 - 1.0 + 2.0 * G2;
                    var n: f32 = 0.0;
                    var tc: f32 = 0.5 - x0*x0 - y0*y0;
                    if (tc > 0.0) {
                        let h = fract(sin(i * 127.1 + j * 311.3) * 43758.5);
                        n += tc*tc*tc*tc * (cos(h * 6.28318) * x0 + sin(h * 6.28318) * y0);
                    }
                    tc = 0.5 - x1*x1 - y1*y1;
                    if (tc > 0.0) {
                        let h = fract(sin((i+i1) * 127.1 + (j+j1) * 311.3) * 43758.5);
                        n += tc*tc*tc*tc * (cos(h * 6.28318) * x1 + sin(h * 6.28318) * y1);
                    }
                    tc = 0.5 - x2*x2 - y2*y2;
                    if (tc > 0.0) {
                        let h = fract(sin((i+1.0) * 127.1 + (j+1.0) * 311.3) * 43758.5);
                        n += tc*tc*tc*tc * (cos(h * 6.28318) * x2 + sin(h * 6.28318) * y2);
                    }
                    result = n * 70.0;
                } else { result = 0.0; }
            }
            case OP_DOMAIN_WARP {
                let a_idx = u32(instr.a);
                let b_idx = u32(instr.b);
                let octaves = i32(instr.c);
                if (a_idx < sp && b_idx < sp) {
                    let vx = stack[a_idx];
                    let vy = stack[b_idx];
                    // compute two value_noise calls as warp offsets
                    let xi0 = floor(vx); let yi0 = floor(vy);
                    let fx0 = vx - xi0; let fy0 = vy - yi0;
                    let wx = mix(mix(sin(xi0*127.1+yi0*311.3), sin((xi0+1.0)*127.1+yi0*311.3), fx0),
                                 mix(sin(xi0*127.1+(yi0+1.0)*311.3), sin((xi0+1.0)*127.1+(yi0+1.0)*311.3), fx0), fy0);
                    let xi1 = floor(vy); let yi1 = floor(vx);
                    let fx1 = vy - xi1; let fy1 = vx - yi1;
                    let wy = mix(mix(sin(xi1*127.1+yi1*311.3), sin((xi1+1.0)*127.1+yi1*311.3), fx1),
                                 mix(sin(xi1*127.1+(yi1+1.0)*311.3), sin((xi1+1.0)*127.1+(yi1+1.0)*311.3), fx1), fy1);
                    // FBM at warped coordinates
                    let wx2 = vx + wx;
                    let wy2 = vy + wy;
                    var value: f32 = 0.0;
                    var amplitude: f32 = 1.0;
                    var frequency: f32 = 1.0;
                    var max_val: f32 = 0.0;
                    for (var o: i32 = 0; o < max(1, min(8, octaves)); o += 1) {
                        let sx = wx2 * frequency; let sy = wy2 * frequency;
                        let xi = floor(sx); let yi = floor(sy);
                        let fx = sx - xi; let fy = sy - yi;
                        let fa = sin(xi * 127.1 + yi * 311.3);
                        let fb = sin((xi + 1.0) * 127.1 + yi * 311.3);
                        let fc = sin(xi * 127.1 + (yi + 1.0) * 311.3);
                        let fd = sin((xi + 1.0) * 127.1 + (yi + 1.0) * 311.3);
                        value += mix(mix(fa, fb, fx), mix(fc, fd, fx), fy) * amplitude;
                        max_val += amplitude;
                        amplitude *= 0.5;
                        frequency *= 2.0;
                    }
                    if (max_val > 0.0) { result = value / max_val; } else { result = 0.0; }
                } else { result = 0.0; }
            }
            case OP_SCALED_X { result = nx * instr.value; }
            case OP_SCALED_Y { result = ny * instr.value; }
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
    let bounded = tanh(raw * 0.05) * 20.0;  // identity for |raw|<10, soft-clamps beyond ±20
    return fract(fract(bounded) + 1.0);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Check if we're in bounds
    if (global_id.x >= output_info.width || global_id.y >= output_info.height) {
        return;
    }

    // Calculate which tile and position within tile using dynamic dimensions
    let tile_w = output_info.tile_w;
    let tile_h = output_info.tile_h;
    let tile_x = global_id.x / tile_w;
    let tile_y = global_id.y / tile_h;
    let local_x = global_id.x % tile_w;
    let local_y = global_id.y % tile_h;

    // Tile index in grid (grid_cols = 4)
    let tile_idx = tile_y * 4u + tile_x;
    let ind_idx = tile_idx;

    // 6 genomes per individual: H, S, V spatial + H, S, V remap
    let h_genome_idx  = ind_idx * 6u;
    let s_genome_idx  = ind_idx * 6u + 1u;
    let v_genome_idx  = ind_idx * 6u + 2u;
    let hr_genome_idx = ind_idx * 6u + 3u;
    let sr_genome_idx = ind_idx * 6u + 4u;
    let vr_genome_idx = ind_idx * 6u + 5u;

    // Normalize coordinates to [-1, 1], applying sub-pixel jitter for multi-pass AA.
    // jitter_x/y are in render-pixel units; 0.0 for non-save renders.
    let nx = (f32(local_x) + output_info.jitter_x) / f32(tile_w) * 2.0 - 1.0;
    let ny = (f32(local_y) + output_info.jitter_y) / f32(tile_h) * 2.0 - 1.0;

    // Stage 1: spatial evaluation (t unused, pass 0.0)
    let raw_h = evaluate(h_genome_idx * INSTRUCTIONS_PER_GENOME, nx, ny, 0.0);
    let raw_s = evaluate(s_genome_idx * INSTRUCTIONS_PER_GENOME, nx, ny, 0.0);
    let raw_v = evaluate(v_genome_idx * INSTRUCTIONS_PER_GENOME, nx, ny, 0.0);

    // Stage 2: palette remap (t = raw channel value; nx/ny unused in pure 1D remaps)
    let h = evaluate(hr_genome_idx * INSTRUCTIONS_PER_GENOME, 0.0, 0.0, raw_h);
    let s = evaluate(sr_genome_idx * INSTRUCTIONS_PER_GENOME, 0.0, 0.0, raw_s);
    let v = evaluate(vr_genome_idx * INSTRUCTIONS_PER_GENOME, 0.0, 0.0, raw_v);

    // Convert to RGB and output — color space selected per-individual by the host.
    let rgb = clamp(channels_to_rgb(output_info.color_model, h, s, v), vec3<f32>(0.0), vec3<f32>(1.0));
    let out_idx = global_id.y * output_info.width + global_id.x;
    output[out_idx] = vec4<f32>(rgb, 1.0);
}
