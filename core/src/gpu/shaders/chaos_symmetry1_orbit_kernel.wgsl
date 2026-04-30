// ChaosSymmetry1 orbit accumulation kernel.
// Formula: z_{n+1} = (a0 + a1*|z|^2 + a2*Re(z^m) + a3*i) * z  +  a4 * conj(z)^(m-1)
//
// Parameter slots (repurposed from the shared OrbitParams struct):
//   map_re[0] = a0  (base scalar term)
//   map_re[1] = a1  (|z|^2 weight)
//   map_re[2] = a2  (Re(z^m) perturbation)
//   map_re[3] = a3  (bilateral-symmetry imaginary offset)
//   map_re[4] = a4  (conjugate term scale)
//   num_maps   = m  (symmetry degree, integer 2-8)
//   map_im, cum_prob — unused

// ── Complex helpers ─────────────────────────────────────────────────────────

fn cs1_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// z^n via repeated multiplication (n is small: 1-8)
fn cs1_powi(z: vec2<f32>, n: u32) -> vec2<f32> {
    var result = vec2<f32>(1.0, 0.0);
    for (var i = 0u; i < n; i++) {
        result = cs1_mul(result, z);
    }
    return result;
}

// ── Orbit kernel ─────────────────────────────────────────────────────────────

fn trace_orbit(rng: ptr<function, u32>, samples: u32, burn_in: u32) {
    let a0 = params.map_re[0];
    let a1 = params.map_re[1];
    let a2 = params.map_re[2];
    let a3 = params.map_re[3];
    let a4 = params.map_re[4];
    let m = max(params.num_maps, 2u);

    // Deterministic starting point: use two RNG draws for angle and radius.
    // This gives each sub-orbit a varied starting position on the attractor,
    // matching the golden-angle spread used in the CPU path.
    let r0 = orbit_xorshift(rng);
    let angle = f32(r0 & 0xFFFFu) * (6.2831853 / 65536.0);
    let r1 = orbit_xorshift(rng);
    let radius = 0.1 + f32(r1 % 7u) * 0.03;
    var z = vec2<f32>(radius * cos(angle), radius * sin(angle));

    // Escape threshold matching the CPU (ESCAPE_SQ = 1e12, so |z| > 1e6)
    let escape_sq = 1e12f;

    let total = samples + burn_in;
    for (var step = 0u; step < total; step++) {
        // z^m and conj(z)^(m-1)
        let mod2     = dot(z, z);
        let zm       = cs1_powi(z, m);
        let re_zm    = zm.x;
        let conj_z   = vec2<f32>(z.x, -z.y);
        let conj_zm1 = cs1_powi(conj_z, m - 1u);

        // factor = (a0 + a1*|z|^2 + a2*Re(z^m))  +  a3*i
        let scale  = a0 + a1 * mod2 + a2 * re_zm;
        let factor = vec2<f32>(scale, a3);

        // z_{n+1} = factor * z + a4 * conj(z)^(m-1)
        let new_z = cs1_mul(factor, z) + a4 * conj_zm1;

        // Guard: reset to origin if escaped or NaN
        if dot(new_z, new_z) > escape_sq || any(new_z != new_z) {
            z = vec2<f32>(0.0, 0.0);
        } else {
            z = new_z;
        }

        // Record orbit point after burn-in
        if step >= burn_in {
            let pixel_idx = complex_to_pixel_index(z);
            if pixel_idx != 0xFFFFFFFFu {
                atomicAdd(&density[pixel_idx], 1u);
            }
        }
    }
}
