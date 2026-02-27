// Zubieta fractal iteration kernel
//
// param_0 = c.real
// param_1 = c.imag
//
// z_{n+1} = z_n^2 + c/z_n,  z_0 = pixel coordinate
// Escapes when |z| > 2
// Guards against division by zero

fn iterate_fractal(pixel: vec2<f32>) -> u32 {
    var z = pixel;
    let c = vec2<f32>(params.param_0, params.param_1);
    let escape_radius_sq = 4.0;  // |z| > 2 means z^2 > 4
    let epsilon_sq = 1e-30;  // Guard against division by zero

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_mag_sq = z.x * z.x + z.y * z.y;
        
        if (z_mag_sq > escape_radius_sq) {
            return i;
        }
        
        // Guard against division by zero
        if (z_mag_sq < epsilon_sq) {
            return i;
        }

        // z_{n+1} = z_n^2 + c/z_n
        // Complex square: z^2
        let z_squared = complex_mul(z, z);
        
        // Complex division: c/z = (c.re*z.re + c.im*z.im, c.im*z.re - c.re*z.im) / (z.re^2 + z.im^2)
        let c_div_z = vec2<f32>(
            (c.x * z.x + c.y * z.y) / z_mag_sq,
            (c.y * z.x - c.x * z.y) / z_mag_sq
        );
        
        z = z_squared + c_div_z;
    }

    return params.max_iter;
}
