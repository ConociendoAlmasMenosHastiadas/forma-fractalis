// Tetration fractal iteration kernel
//
// param_0 = threshold  (escape threshold, default 1e7, range 10..1e10)
// param_1 = escape_mode (0=Magnitude, 1=Real, 2=Imaginary, 3=Either)
//
// z_{n+1} = c^{z_n} = exp(z_n * ln(c))
// z_0 = c  (pixel coordinate)
//
// NOTE: This kernel defines complex_ln_tet, complex_exp_tet, and complex_cpow
// with unique names to avoid collision with complex_pow (scalar power) in common.wgsl.

// Natural logarithm of a complex number z: ln(z) = log(|z|) + i*arg(z)
// Guard against z = 0 (undefined): return a large negative real (signals near-zero c).
fn complex_ln_tet(z: vec2<f32>) -> vec2<f32> {
    let r = length(z);
    if r < 1e-30 {
        // ln(0) is -inf. Return (-69, 0) which is approx ln(1e-30).
        // exp(-69 * anything finite) = ~0, keeping iteration bounded.
        return vec2<f32>(-69.0, 0.0);
    }
    return vec2<f32>(log(r), atan2(z.y, z.x));
}

// Complex exponential: exp(z) = e^(Re(z)) * (cos(Im(z)) + i*sin(Im(z)))
fn complex_exp_tet(z: vec2<f32>) -> vec2<f32> {
    let e_re = exp(z.x);
    return vec2<f32>(e_re * cos(z.y), e_re * sin(z.y));
}

// Complex base raised to a complex exponent: base^exp = exp(exp * ln(base))
// Named complex_cpow to distinguish from complex_pow(z, scalar) in common.wgsl.
fn complex_cpow(base: vec2<f32>, exponent: vec2<f32>) -> vec2<f32> {
    return complex_exp_tet(complex_mul(exponent, complex_ln_tet(base)));
}

// Escape test supporting all four modes
fn tet_escaped(z: vec2<f32>, threshold: f32, mode: u32) -> bool {
    if mode == 0u { return dot(z, z) > threshold * threshold; }  // Magnitude
    if mode == 1u { return abs(z.x) > threshold; }               // Real
    if mode == 2u { return abs(z.y) > threshold; }               // Imaginary
    if mode == 3u { return abs(z.x) > threshold || abs(z.y) > threshold; }  // Either
    // Fallback: Magnitude
    return dot(z, z) > threshold * threshold;
}

fn iterate_fractal(c: vec2<f32>) -> u32 {
    let threshold  = params.param_0;
    let escape_mode = u32(params.param_1);

    // Safety ceiling: if z exceeds this we stop regardless of escape mode.
    // Prevents f32 overflow to Inf from silently bypassing Real/Imag checks.
    let safety_sq = 1e30;

    var z = c;  // z_0 = c

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        // Primary escape check
        if tet_escaped(z, threshold, escape_mode) {
            return i;
        }

        // Safety: catch overflow to Inf before NaN can propagate
        if dot(z, z) > safety_sq {
            return i;
        }

        // z_{n+1} = c^{z_n}
        z = complex_cpow(c, z);
    }

    return params.max_iter;
}
