// Mandelbrot Set Compute Shader
// Computes iteration counts for each pixel in parallel

struct FractalParams {
    center_x: f32,
    center_y: f32,
    zoom: f32,
    max_iter: u32,
    width: u32,
    height: u32,
    power: f32,      // Mandelbrot power (typically 2.0)
    _padding: u32,   // Align to 16 bytes
}

@group(0) @binding(0)
var<uniform> params: FractalParams;

@group(0) @binding(1)
var<storage, read_write> output: array<u32>;

// Convert pixel coordinates to complex plane coordinates
fn pixel_to_complex(pixel_x: u32, pixel_y: u32) -> vec2<f32> {
    let aspect = f32(params.width) / f32(params.height);
    
    // Normalize pixel coordinates to [0, 1]
    let norm_x = f32(pixel_x) / f32(params.width);
    let norm_y = f32(pixel_y) / f32(params.height);
    
    // Map to complex plane - matches CPU screen_to_complex exactly:
    // scale = 3.5 / zoom, range_x = scale * aspect, range_y = scale
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
// Using: z^n = r^n * (cos(n*theta) + i*sin(n*theta))
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

// Mandelbrot iteration: z_{n+1} = z_n^power + c
// Returns iteration count when |z| > 2, or max_iter if bounded
fn iterate_mandelbrot(c: vec2<f32>) -> u32 {
    var z = vec2<f32>(0.0, 0.0);
    let escape_radius_sq = 4.0;  // |z| > 2 means z^2 > 4
    
    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        // Check escape condition
        let z_mag_sq = z.x * z.x + z.y * z.y;
        if (z_mag_sq > escape_radius_sq) {
            return i;
        }
        
        // z = z^power + c
        if (params.power == 2.0) {
            // Optimized path for standard Mandelbrot (power = 2)
            z = complex_mul(z, z) + c;
        } else {
            // General power path
            z = complex_pow(z, params.power) + c;
        }
    }
    
    return params.max_iter;  // Point is in the set (or took too long to escape)
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_x = global_id.x;
    let pixel_y = global_id.y;
    
    // Bounds check
    if (pixel_x >= params.width || pixel_y >= params.height) {
        return;
    }
    
    // Convert pixel to complex coordinates
    let c = pixel_to_complex(pixel_x, pixel_y);
    
    // Compute iteration count
    let iter = iterate_mandelbrot(c);
    
    // Write to output buffer (row-major order)
    let idx = pixel_y * params.width + pixel_x;
    output[idx] = iter;
}
