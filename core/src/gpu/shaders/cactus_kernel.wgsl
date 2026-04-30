// Cactus fractal iteration kernel
//
// No fractal-specific params used.
//
// z_{n+1} = z_n^3 + (z_0 - 1)*z_n - z_0,  z_0 = pixel coordinate
// Escape radius is per-pixel: R = 2 * max(1, sqrt(|z_0 - 1| + 1), |z_0|^(1/3))
//
// Note: z_0 is also the pixel coordinate c, so this is an autonomous iteration
// where every pixel uses its own escape threshold.

fn iterate_fractal(c: vec2<f32>) -> u32 {
    // Compute per-pixel escape radius
    // r1 = 1
    // r2 = sqrt(|c - 1| + 1)
    // r3 = |c|^(1/3)
    let c_minus_1 = vec2<f32>(c.x - 1.0, c.y);
    let c_minus_1_norm = sqrt(c_minus_1.x * c_minus_1.x + c_minus_1.y * c_minus_1.y);
    let r2 = sqrt(c_minus_1_norm + 1.0);

    let c_norm = sqrt(c.x * c.x + c.y * c.y);
    // |c|^(1/3): use exp(log/3) with guard for zero
    let r3 = select(pow(c_norm, 1.0 / 3.0), 0.0, c_norm < 1e-30);

    let escape_radius = 2.0 * max(1.0, max(r2, r3));
    let escape_radius_sq = escape_radius * escape_radius;

    // z_0 = c (the pixel coordinate)
    var z = c;

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_mag_sq = z.x * z.x + z.y * z.y;
        if (z_mag_sq > escape_radius_sq) {
            return i;
        }

        // z^3 via two complex multiplications
        let z2 = complex_mul(z, z);
        let z3 = complex_mul(z2, z);

        // (c - 1) * z
        let c_minus1_times_z = complex_mul(vec2<f32>(c.x - 1.0, c.y), z);

        // z_{n+1} = z^3 + (c - 1)*z - c
        z = z3 + c_minus1_times_z - c;
    }

    return params.max_iter;
}
