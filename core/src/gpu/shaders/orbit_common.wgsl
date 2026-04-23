// Orbit accumulation shader template.
// Per-fractal orbit logic is injected at the {{ORBIT_KERNEL}} marker.
//
// Architecture: 1D dispatch, each thread runs one sub-orbit of
// `params.samples_per_thread` steps, writing to a shared atomic density buffer.

struct OrbitParams {
    center_x: f32,
    center_y: f32,
    zoom: f32,
    width: u32,
    height: u32,
    samples_per_thread: u32,
    burn_in: u32,
    seed_base: u32,
    num_maps: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    // Map data: 8 maps x (c_real, c_imag) = 16 floats
    map_re: array<f32, 8>,
    map_im: array<f32, 8>,
    // Cumulative probabilities for map selection
    cum_prob: array<f32, 8>,
}

@group(0) @binding(0) var<storage, read> params: OrbitParams;
@group(0) @binding(1) var<storage, read_write> density: array<atomic<u32>>;

// ── PRNG ───────────────────────────────────────────────────────────────────

fn orbit_xorshift(state: ptr<function, u32>) -> u32 {
    var s = *state;
    s ^= s << 13u;
    s ^= s >> 17u;
    s ^= s << 5u;
    *state = s;
    return s;
}

// ── Complex math ───────────────────────────────────────────────────────────

fn complex_sqrt_wgsl(w: vec2<f32>) -> vec2<f32> {
    let r = length(w);
    if r < 1e-30 {
        return vec2<f32>(0.0, 0.0);
    }
    let sqrt_r = sqrt(r);
    let half_theta = atan2(w.y, w.x) * 0.5;
    return vec2<f32>(sqrt_r * cos(half_theta), sqrt_r * sin(half_theta));
}

// ── Coordinate mapping ─────────────────────────────────────────────────────

// Map a complex-plane point to a pixel index. Returns 0xFFFFFFFF if out of bounds.
fn complex_to_pixel_index(z: vec2<f32>) -> u32 {
    let scale = 3.5 / params.zoom;
    let aspect = f32(params.width) / f32(params.height);
    let range_x = scale * aspect;
    let range_y = scale;

    let px_f = (z.x - params.center_x) * f32(params.width) / range_x + f32(params.width) * 0.5;
    let py_f = (z.y - params.center_y) * f32(params.height) / range_y + f32(params.height) * 0.5;

    if px_f < 0.0 || py_f < 0.0 {
        return 0xFFFFFFFFu;
    }
    let px = u32(px_f);
    let py = u32(py_f);
    if px >= params.width || py >= params.height {
        return 0xFFFFFFFFu;
    }
    return py * params.width + px;
}

// ── Map selection ──────────────────────────────────────────────────────────

fn choose_map(rng: ptr<function, u32>) -> u32 {
    let r_bits = orbit_xorshift(rng);
    let t = f32(r_bits) / 4294967295.0; // normalize to [0, 1]
    for (var i = 0u; i < params.num_maps; i++) {
        if t <= params.cum_prob[i] {
            return i;
        }
    }
    return params.num_maps - 1u;
}

// {{ORBIT_KERNEL}}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let thread_id = gid.x;
    // Each thread is one sub-orbit
    var seed = params.seed_base + thread_id * 2654435761u; // golden ratio mixing
    if seed == 0u {
        seed = 0xDEADBEEFu;
    }

    trace_orbit(&seed, params.samples_per_thread, params.burn_in);
}
