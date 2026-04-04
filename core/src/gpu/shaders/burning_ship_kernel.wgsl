// Burning Ship iteration kernel
//
// No fractal-specific params used.
//
// z_{n+1} = (|Re(z_n)| + i|Im(z_n)|)^2 + c,  z_0 = 0
// Escapes when |z|^2 > 4

fn iterate_fractal(c: vec2<f32>) -> u32 {
    var z = vec2<f32>(0.0, 0.0);
    let escape_radius_sq = 4.0;

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
