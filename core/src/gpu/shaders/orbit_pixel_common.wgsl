// Per-screen-seed orbit accumulation shader template.
//
// Each invocation owns one screen pixel as its initial seed and traces that
// seed's orbit, writing visit counts to a shared atomic density histogram.

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
    map_re: array<f32, 8>,
    map_im: array<f32, 8>,
    cum_prob: array<f32, 8>,
}

@group(0) @binding(0) var<storage, read> params: OrbitParams;
@group(0) @binding(1) var<storage, read_write> density: array<atomic<u32>>;

fn pixel_to_complex(pixel_x: u32, pixel_y: u32) -> vec2<f32> {
    let aspect = f32(params.width) / f32(params.height);
    let norm_x = f32(pixel_x) / f32(params.width);
    let norm_y = f32(pixel_y) / f32(params.height);
    let scale = 3.5 / params.zoom;
    let range_x = scale * aspect;
    let range_y = scale;

    let real = params.center_x + (norm_x - 0.5) * range_x;
    let imag = params.center_y + (norm_y - 0.5) * range_y;
    return vec2<f32>(real, imag);
}

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

// {{ORBIT_PIXEL_KERNEL}}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let pixel_x = gid.x;
    let pixel_y = gid.y;

    if pixel_x >= params.width || pixel_y >= params.height {
        return;
    }

    trace_seed(vec2<u32>(pixel_x, pixel_y));
}