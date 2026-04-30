// Burning Ship iteration kernel
//
// param_0 = escape_radius (default 2.0)
//
// z_{n+1} = (|Re(z_n)| + i|Im(z_n)|)^2 + c,  z_0 = 0
// Escapes when |z| > escape_radius

fn iterate_fractal(c: vec2<f32>) -> u32 {
    var z = vec2<f32>(0.0, 0.0);
    let escape_r = params.param_0;
    let escape_radius_sq = escape_r * escape_r;

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_mag_sq = z.x * z.x + z.y * z.y;
        if (z_mag_sq > escape_radius_sq) {
            return i;
        }

        // Take absolute value of both components before squaring
        let z_abs = vec2<f32>(abs(z.x), abs(z.y));
        z = complex_mul(z_abs, z_abs) + c;
    }

    return params.max_iter;
}
