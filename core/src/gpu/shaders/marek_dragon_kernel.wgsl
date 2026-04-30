// Marek Dragon fractal iteration kernel
//
// param_0 = phi (rotation angle, 0 to 2*pi)
// param_1 = escape_radius (default 2.0)
//
// z_{n+1} = exp(j*phi)*z_n + z_n^2,  z_0 = pixel coordinate c
// exp(j*phi) = cos(phi) + j*sin(phi)
// Escapes when |z| > escape_radius

fn iterate_fractal(c: vec2<f32>) -> u32 {
    let phi = params.param_0;

    // Precompute the rotation factor: exp(j*phi) = (cos(phi), sin(phi))
    let rotation = vec2<f32>(cos(phi), sin(phi));

    // z starts at the pixel coordinate
    var z = c;
    let escape_r = params.param_1;
    let escape_radius_sq = escape_r * escape_r;

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_mag_sq = z.x * z.x + z.y * z.y;
        if (z_mag_sq > escape_radius_sq) {
            return i;
        }

        // z_{n+1} = exp(j*phi)*z + z^2
        let rotated = complex_mul(rotation, z);
        let z_sq = complex_mul(z, z);
        z = rotated + z_sq;
    }

    return params.max_iter;
}
