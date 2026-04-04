// Julia Set iteration kernel
//
// param_0 = c_real (real part of Julia constant)
// param_1 = c_imag (imaginary part of Julia constant)
// param_2 = power  (exponent k in z^k + c, default 2.0)
//
// z_{n+1} = z_n^k + c,  z_0 = pixel coordinate
// Escapes when |z| > 2
//
// Note: Unlike Mandelbrot, the pixel coordinate becomes z_0,
// and c is a fixed constant from parameters.

fn iterate_fractal(pixel_coord: vec2<f32>) -> u32 {
    // Julia constant from parameters
    let c = vec2<f32>(params.param_0, params.param_1);
    let power = params.param_2;

    // z starts at the pixel coordinate (not at origin like Mandelbrot)
    var z = pixel_coord;

    let escape_radius_sq = 4.0;  // |z| > 2 means z^2 > 4

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_mag_sq = z.x * z.x + z.y * z.y;
        if (z_mag_sq > escape_radius_sq) {
            return i;
        }

        // z = z^power + c  (fast path for power=2, general polar form otherwise)
        if (power == 2.0) {
            z = complex_mul(z, z) + c;
        } else {
            z = complex_pow(z, power) + c;
        }
    }

    return params.max_iter;
}
