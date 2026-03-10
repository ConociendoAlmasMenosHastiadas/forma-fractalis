// Sin Julia Set iteration kernel
//
// param_0 = c_real (real part of Sin Julia constant)
// param_1 = c_imag (imaginary part of Sin Julia constant)
// param_2 = escape_radius (escape boundary, default 50.0)
//
// z_{n+1} = c * sin(z_n),  z_0 = pixel coordinate
// Escapes when |z| > escape_radius
//
// Iteration formula (simplified form):
// x_{n+1} = sin(x_n) * cosh(y_n)
// y_{n+1} = cos(x_n) * sinh(y_n)
// Then multiply by c
//
// Note: Like Julia, the pixel coordinate becomes z_0,
// and c is a fixed constant from parameters.

fn iterate_fractal(pixel_coord: vec2<f32>) -> u32 {
    // Sin Julia constant from parameters
    let c = vec2<f32>(params.param_0, params.param_1);
    let escape_radius = params.param_2;
    
    // z starts at the pixel coordinate
    var z = pixel_coord;
    
    // Escape radius from parameter
    let escape_radius_sq = escape_radius * escape_radius;

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_mag_sq = z.x * z.x + z.y * z.y;
        if (z_mag_sq > escape_radius_sq) {
            return i;
        }

        // Compute sin(z) = sin(x + iy)
        // sin(x + iy) = sin(x)cosh(y) + i*cos(x)sinh(y)
        let sin_x = sin(z.x);
        let cos_x = cos(z.x);
        let sinh_y = sinh(z.y);
        let cosh_y = cosh(z.y);
        
        let sin_z = vec2<f32>(
            sin_x * cosh_y,
            cos_x * sinh_y
        );
        
        // Multiply by c: z_{n+1} = c * sin(z_n)
        z = complex_mul(c, sin_z);
    }

    return params.max_iter;
}
