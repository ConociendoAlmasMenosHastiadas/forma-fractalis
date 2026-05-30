// Wallpaper orbit accumulation kernel.
// Formula:
//   x_{n+1} = y_n - sign(x_n) * sqrt(abs(b*x_n - c))
//   y_{n+1} = a - x_n
//
// Parameter slots:
//   map_re[0] = a
//   map_re[1] = b
//   map_re[2] = c
//   samples_per_thread = samples per seed
//   burn_in = burn-in steps per seed

fn trace_seed(seed_pixel: vec2<u32>) {
    let a = params.map_re[0];
    let b = params.map_re[1];
    let c = params.map_re[2];
    let escape_limit = 1e12f;

    var z = pixel_to_complex(seed_pixel.x, seed_pixel.y);
    let total = params.samples_per_thread + params.burn_in;

    for (var step = 0u; step < total; step++) {
        let root = sqrt(abs(b * z.x - c));

        var next_x = z.y;
        if z.x > 0.0 {
            next_x = z.y - root;
        } else if z.x < 0.0 {
            next_x = z.y + root;
        }

        let next = vec2<f32>(next_x, a - z.x);
        if any(abs(next) > vec2<f32>(escape_limit)) || any(next != next) {
            break;
        }

        z = next;

        if step >= params.burn_in {
            let pixel_idx = complex_to_pixel_index(z);
            if pixel_idx != 0xFFFFFFFFu {
                atomicAdd(&density[pixel_idx], 1u);
            }
        }
    }
}