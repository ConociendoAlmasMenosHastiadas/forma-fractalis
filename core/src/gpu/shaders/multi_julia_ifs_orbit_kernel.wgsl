// Multi-Julia IFS orbit kernel.
// Implements the chaos game: z = ±sqrt(z - c_i) with random map selection.
//
// Param slots:
//   map_re[0..7], map_im[0..7] — attractor complex coordinates
//   cum_prob[0..7] — cumulative selection probabilities
//   num_maps — number of active maps (2-8)

fn trace_orbit(rng: ptr<function, u32>, samples: u32, burn_in: u32) {
    var z = vec2<f32>(0.0, 0.0);

    let total = samples + burn_in;
    for (var step = 0u; step < total; step++) {
        // Choose random map
        let idx = choose_map(rng);

        // w = z - c_i
        let c = vec2<f32>(params.map_re[idx], params.map_im[idx]);
        let w = z - c;

        // s = sqrt(w)
        var s = complex_sqrt_wgsl(w);

        // Randomly negate (choose branch)
        if (orbit_xorshift(rng) & 1u) == 1u {
            s = -s;
        }

        // Guard NaN/Inf
        if any(abs(s) > vec2<f32>(1e30)) || any(s != s) {
            z = vec2<f32>(0.0, 0.0);
        } else {
            z = s;
        }

        // Record after burn-in
        if step >= burn_in {
            let pixel_idx = complex_to_pixel_index(z);
            if pixel_idx != 0xFFFFFFFFu {
                atomicAdd(&density[pixel_idx], 1u);
            }
        }
    }
}
