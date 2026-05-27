// Lemon fractal iteration kernel
//
// param_0 = denom_power (k in (z^2 - 1)^k, default 2.0)
// param_1 = convergence_exp (threshold = 10^(-exp), default 6.0)
//
// z_{n+1} = z_0 * z_n^2 * (z_n^2 + 1) / (z_n^2 - 1)^k, z_0 = pixel coordinate c
// Converges when |z_{n+1} - z_n| < threshold

fn complex_div_lemon(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let denom = dot(b, b);
    return vec2<f32>(
        (a.x * b.x + a.y * b.y) / denom,
        (a.y * b.x - a.x * b.y) / denom,
    );
}

fn complex_is_finite_lemon(z: vec2<f32>) -> bool {
    let finite_limit = vec2<f32>(3.4028235e38, 3.4028235e38);
    return all(z == z) && all(abs(z) < finite_limit);
}

fn iterate_fractal(c: vec2<f32>) -> u32 {
    let denom_power = params.param_0;
    let convergence_exp = params.param_1;
    let threshold = pow(10.0, -convergence_exp);
    let denom_base_guard = 1e-12;
    let denominator_guard = 1e-24;
    let one = vec2<f32>(1.0, 0.0);

    let z0 = c;
    var z = c;

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_sq = complex_mul(z, z);
        let numerator = complex_mul(complex_mul(z0, z_sq), z_sq + one);
        let denom_base = z_sq - one;

        if (dot(denom_base, denom_base) < denom_base_guard) {
            return i;
        }

        var denominator = complex_pow(denom_base, denom_power);
        if (denom_power == 1.0) {
            denominator = denom_base;
        } else if (denom_power == 2.0) {
            denominator = complex_mul(denom_base, denom_base);
        }

        if (dot(denominator, denominator) < denominator_guard) {
            return i;
        }

        let z_next = complex_div_lemon(numerator, denominator);
        if (!complex_is_finite_lemon(z_next)) {
            return i;
        }

        let delta = z_next - z;
        if (length(delta) < threshold) {
            return i;
        }

        if (dot(z_next, z_next) > 1e30) {
            return i;
        }

        z = z_next;
    }

    return params.max_iter;
}