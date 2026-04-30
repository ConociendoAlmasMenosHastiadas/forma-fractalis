// Lace Julia fractal iteration kernel
//
// param_0 = c_real  (real part of Julia constant c)
// param_1 = c_imag  (imaginary part of Julia constant c)
// param_2 = escape_radius (default 2.0)
//
// Formula (multiplied through by z^6):
//   z_{n+1} = (i*z_n^3 + 1010*z_n^6) / (c*i + 3301*z_n^7)
//
// z_0 = pixel coordinate
// Escapes when |z| > escape_radius
//
// Singularity guards:
//   - |z|^2 < 1e-20: z is at a pole, treat as escaped
//   - |denominator|^2 < 1e-20: treat as escaped

fn iterate_fractal(pixel: vec2<f32>) -> u32 {
    let c = vec2<f32>(params.param_0, params.param_1);
    // c * i = (-c.y, c.x)
    let c_times_i = vec2<f32>(-c.y, c.x);

    let escape_r    = params.param_2;
    let escape_sq   = escape_r * escape_r;
    let epsilon_sq  = 1e-20;

    var z = pixel;

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_sq = z.x * z.x + z.y * z.y;

        // Pole guard
        if (z_sq < epsilon_sq) {
            return i;
        }
        // Escape check
        if (z_sq > escape_sq) {
            return i;
        }

        // z^2, z^3, z^6, z^7 via repeated complex_mul
        let z2 = complex_mul(z,  z);
        let z3 = complex_mul(z2, z);
        let z6 = complex_mul(z3, z3);
        let z7 = complex_mul(z6, z);

        // Numerator: i*z3 + 1010*z6
        // i*(a,b) = (-b, a)
        let i_z3  = vec2<f32>(-z3.y, z3.x);
        let numer = i_z3 + 1010.0 * z6;

        // Denominator: c*i + 3301*z7
        let denom = c_times_i + 3301.0 * z7;

        let denom_sq = denom.x * denom.x + denom.y * denom.y;
        if (denom_sq < epsilon_sq) {
            return i;
        }

        // Complex division: numer / denom
        let inv = 1.0 / denom_sq;
        z = vec2<f32>(
            (numer.x * denom.x + numer.y * denom.y) * inv,
            (numer.y * denom.x - numer.x * denom.y) * inv
        );
    }

    return params.max_iter;
}
