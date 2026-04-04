// Common Fractal Compute Shader Framework
//
// Shared infrastructure for all GPU fractal shaders.
// Fractal-specific iteration kernels are injected at the {{FRACTAL_KERNEL}} marker.
//
// CONTRACT: Each fractal kernel must define:
//   fn iterate_fractal(c: vec2<f32>) -> u32
//
// Available to kernels:
//   - params.param_0, params.param_1, params.param_2 (per-fractal parameters)
//   - params.max_iter, params.width, params.height, params.zoom
//   - pixel_to_complex(), complex_mul(), complex_pow(), sinh(), cosh()

struct FractalParams {
    center_x: f32,
    center_y: f32,
    zoom: f32,
    max_iter: u32,
    width: u32,
    height: u32,
    param_0: f32,   // Per-fractal parameter slot 0
    param_1: f32,   // Per-fractal parameter slot 1
    param_2: f32,   // Per-fractal parameter slot 2
}

@group(0) @binding(0)
var<uniform> params: FractalParams;

@group(0) @binding(1)
var<storage, read_write> output: array<u32>;

// Convert pixel coordinates to complex plane coordinates
// Matches CPU screen_to_complex exactly:
//   scale = 3.5 / zoom, range_x = scale * aspect, range_y = scale
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

// Complex number multiplication: (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
fn complex_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        a.x * b.x - a.y * b.y,
        a.x * b.y + a.y * b.x
    );
}

// Complex number power for general exponent
// Using polar form: z^n = r^n * (cos(n*theta) + i*sin(n*theta))
fn complex_pow(z: vec2<f32>, power: f32) -> vec2<f32> {
    let r = length(z);
    if (r < 1e-10) {
        return vec2<f32>(0.0, 0.0);
    }

    let theta = atan2(z.y, z.x);
    let r_pow = pow(r, power);
    let theta_pow = theta * power;

    return vec2<f32>(
        r_pow * cos(theta_pow),
        r_pow * sin(theta_pow)
    );
}

// Hyperbolic sine: sinh(x) = (e^x - e^(-x)) / 2
fn sinh(x: f32) -> f32 {
    let ex = exp(x);
    return (ex - 1.0 / ex) * 0.5;
}

// Hyperbolic cosine: cosh(x) = (e^x + e^(-x)) / 2
fn cosh(x: f32) -> f32 {
    let ex = exp(x);
    return (ex + 1.0 / ex) * 0.5;
}

// --- FRACTAL KERNEL INSERTED HERE ---
// {{FRACTAL_KERNEL}}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_x = global_id.x;
    let pixel_y = global_id.y;

    // Bounds check for workgroups that extend past image edges
    if (pixel_x >= params.width || pixel_y >= params.height) {
        return;
    }

    // Convert pixel to complex coordinates
    let c = pixel_to_complex(pixel_x, pixel_y);

    // Compute iteration count (defined by fractal kernel)
    let iter = iterate_fractal(c);

    // Write to output buffer (row-major order)
    let idx = pixel_y * params.width + pixel_x;
    output[idx] = iter;
}
