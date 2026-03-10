// Julia Set iteration kernel
//
// param_0 = c_real (real part of Julia constant)
// param_1 = c_imag (imaginary part of Julia constant)
//
// z_{n+1} = z_n^2 + c,  z_0 = pixel coordinate
// Escapes when |z| > 2
//
// Note: Unlike Mandelbrot, the pixel coordinate becomes z_0,
// and c is a fixed constant from parameters.

fn iterate_fractal(pixel_coord: vec2<f32>) -> u32 {
    // Julia constant from parameters
    let c = vec2<f32>(params.param_0, params.param_1);
    
    // z starts at the pixel coordinate (not at origin like Mandelbrot)
    var z = pixel_coord;
    
    let escape_radius_sq = 4.0;  // |z| > 2 means z^2 > 4

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_mag_sq = z.x * z.x + z.y * z.y;
        if (z_mag_sq > escape_radius_sq) {
            return i;
        }

        // z = z^2 + c
        z = complex_mul(z, z) + c;
    }

    return params.max_iter;
}
