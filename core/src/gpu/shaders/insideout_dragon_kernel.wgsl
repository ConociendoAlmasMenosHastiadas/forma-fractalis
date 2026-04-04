// Insideout Dragon iteration kernel
//
// param_0 = escape_radius (default 4.0)
//
// z_{n+1} = z_n^2 + f(|z_n|) + i*g(|z_n|)
// z_0 = 1/(x + i*y)  (inverse of pixel coordinates)

// Compute both f(r) and g(r) together for better numerical stability.
// Both share the same denominator (1 + r^3)^2, so we compute it once.
//
// f(r) = r*(r+1)^3*(r-1) / (1+r^3)^2
// g(r) = r*(r-1)^3*(r+1) / (1+r^3)^2
fn compute_f_g(r: f32) -> vec2<f32> {
    if (abs(r) < 1e-10) {
        return vec2<f32>(0.0, 0.0);
    }

    let r2 = r * r;
    let r3 = r2 * r;
    let r_plus_1 = r + 1.0;
    let r_minus_1 = r - 1.0;

    let one_plus_r3 = 1.0 + r3;
    let denominator = one_plus_r3 * one_plus_r3;

    if (abs(denominator) < 1e-10) {
        return vec2<f32>(0.0, 0.0);
    }

    let r_plus_1_cubed = r_plus_1 * r_plus_1 * r_plus_1;
    let f_numerator = r * r_plus_1_cubed * r_minus_1;

    let r_minus_1_cubed = r_minus_1 * r_minus_1 * r_minus_1;
    let g_numerator = r * r_minus_1_cubed * r_plus_1;

    let inv_denominator = 1.0 / denominator;
    return vec2<f32>(f_numerator * inv_denominator, g_numerator * inv_denominator);
}

fn iterate_fractal(c: vec2<f32>) -> u32 {
    // Initial condition: z_0 = 1/(x + i*y)
    // 1/(a+bi) = (a-bi)/(a^2+b^2)
    let denom = c.x * c.x + c.y * c.y;

    if (denom < 1e-10) {
        return params.max_iter;
    }

    var z = vec2<f32>(
        c.x / denom,
        -c.y / denom
    );

    let escape_radius_sq = params.param_0 * params.param_0;

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_mag_sq = z.x * z.x + z.y * z.y;
        if (z_mag_sq > escape_radius_sq) {
            return i;
        }

        let magnitude = sqrt(z_mag_sq);
        let f_g = compute_f_g(magnitude);

        let z_squared = complex_mul(z, z);
        z = vec2<f32>(
            z_squared.x + f_g.x,
            z_squared.y + f_g.y
        );
    }

    return params.max_iter;
}
