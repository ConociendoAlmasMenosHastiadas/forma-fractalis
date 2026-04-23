// Multifractal-Julia iteration kernel
//
// param_0 = power k  (exponent in c^k; default 1.0)
//
// Formula: z_{n+1} = c^k * z_n^{-2} + c,   z_0 = c
//
// where z^{-2} = conjugate(z^2) / |z^2|^2
//
// Returns: escape iteration, approximate cycle period for periodic orbits,
//          or max_iter for non-escaping points.
//
// Cycle detection: Brent's algorithm with f32 epsilon.
// NOTE: The CPU implementation uses exact f64 bit-pattern matching (HashMap).
// GPU results for interior periodic pixels will differ from CPU due to f32
// precision — this is expected and acceptable.

fn iterate_fractal(c: vec2<f32>) -> u32 {
    let k = params.param_0;

    // Larger bailout when |k| > 2 (matches CPU bailout_squared logic)
    let bailout_sq = select(16.0, 100.0, abs(k) > 2.0);
    let near_zero = 1e-30;

    // c near zero: avoid singularity in c^k
    if (dot(c, c) < near_zero) {
        return 0u;
    }

    // Precompute c^k — special-case common integer values for numerical stability
    var c_pow_k: vec2<f32>;
    if (k == 0.0) {
        c_pow_k = vec2<f32>(1.0, 0.0);         // c^0 = 1
    } else if (k == 1.0) {
        c_pow_k = c;                             // c^1 = c
    } else if (k == 2.0) {
        c_pow_k = complex_mul(c, c);            // c^2
    } else if (k == -1.0) {
        let c_mag_sq = dot(c, c);
        c_pow_k = vec2<f32>(c.x, -c.y) / c_mag_sq;  // c^{-1} = conj(c)/|c|^2
    } else {
        c_pow_k = complex_pow(c, k);            // general: polar form
    }

    var z = c;

    // Brent's cycle detection with epsilon comparison (f32-safe approximation)
    // When hare (z) revisits tortoise (z_ref) within epsilon, return the
    // approximate cycle length (brent_lam + 1).
    var z_ref = c;
    var brent_power = 1u;
    var brent_lam = 0u;
    let cycle_eps_sq = 1e-8;  // |z - z_ref|^2 threshold

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        // Escape check on current z
        if (dot(z, z) > bailout_sq) {
            return i;
        }

        // Compute z^{-2} = conj(z^2) / |z^2|^2
        let z_sq = complex_mul(z, z);
        let z_sq_mag_sq = dot(z_sq, z_sq);
        if (z_sq_mag_sq < near_zero) {
            // z near zero: denominator underflow, return current iteration
            return i;
        }
        let z_inv_sq = vec2<f32>(z_sq.x, -z_sq.y) / z_sq_mag_sq;

        // z = c^k * z^{-2} + c
        z = complex_mul(c_pow_k, z_inv_sq) + c;

        // Overflow guard: treat very large values as diverged
        if (z.x > 1e30 || z.x < -1e30 || z.y > 1e30 || z.y < -1e30) {
            return i;
        }

        // Brent's cycle check: is new z close to our reference point?
        let dz = z - z_ref;
        if (dot(dz, dz) < cycle_eps_sq) {
            return brent_lam + 1u;
        }

        // Advance Brent's reference at power-of-2 step boundaries
        brent_lam = brent_lam + 1u;
        if (brent_lam >= brent_power) {
            z_ref = z;
            brent_power = brent_power << 1u;
            brent_lam = 0u;
        }
    }

    return params.max_iter;
}
