// Mandelbrot / Powerbrot iteration kernel
//
// param_0 = power (typically 2.0 for standard Mandelbrot)
//
// z_{n+1} = z_n^power + c,  z_0 = 0
// Escapes when |z| > 2

fn iterate_fractal(c: vec2<f32>) -> u32 {
    var z = vec2<f32>(0.0, 0.0);
    let escape_radius_sq = 4.0;  // |z| > 2 means z^2 > 4

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_mag_sq = z.x * z.x + z.y * z.y;
        if (z_mag_sq > escape_radius_sq) {
            return i;
        }

        // z = z^power + c
        if (params.param_0 == 2.0) {
            // Optimized path for standard Mandelbrot (power = 2)
            z = complex_mul(z, z) + c;
        } else {
            // General power path
            z = complex_pow(z, params.param_0) + c;
        }
    }

    return params.max_iter;
}
