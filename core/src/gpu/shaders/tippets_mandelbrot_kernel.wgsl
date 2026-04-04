// Tippets Mandelbrot iteration kernel
//
// No extra parameters needed.
//
// z_0 = (0, 0),  c = (a, b) = pixel coordinate
// Tippets formula uses sequential scalar updates:
//   x = x^2 - y^2 + a       (x updated first with old x, old y)
//   y = 2*x*y + b            (y uses the NEW x just computed above)
// This differs from standard Mandelbrot where both components use the old values.
// Escapes when x^2 + y^2 > 4

fn iterate_fractal(c: vec2<f32>) -> u32 {
    let a = c.x;
    let b = c.y;
    var x = 0.0f;
    var y = 0.0f;
    let escape_radius_sq = 4.0;

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        if (x * x + y * y > escape_radius_sq) {
            return i;
        }

        // Tippets sequential update: x first, then y uses the updated x
        let x_new = x * x - y * y + a;
        y = 2.0 * x_new * y + b;  // uses new x!
        x = x_new;
    }

    return params.max_iter;
}
