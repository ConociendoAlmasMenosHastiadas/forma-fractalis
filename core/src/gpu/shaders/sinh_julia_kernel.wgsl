// Sinh Julia iteration kernel
//
// param_0 = c_real (real part of Julia constant)
// param_1 = c_imag (imaginary part of Julia constant)
// param_2 = escape_radius (escape boundary, default 50.0)
//
// z_{n+1} = abs(sinh(z_n)^4) + c
// abs() is applied independently to the real and imaginary components.

fn complex_is_finite_sinh_julia(z: vec2<f32>) -> bool {
    let max_finite = 3.4028235e38;
    return abs(z.x) <= max_finite && abs(z.y) <= max_finite;
}

fn iterate_fractal(pixel_coord: vec2<f32>) -> u32 {
    let c = vec2<f32>(params.param_0, params.param_1);
    let escape_radius = params.param_2;
    let escape_radius_sq = escape_radius * escape_radius;

    var z = pixel_coord;

    for (var i = 0u; i < params.max_iter; i = i + 1u) {
        let z_mag_sq = dot(z, z);
        if (z_mag_sq > escape_radius_sq) {
            return i;
        }

        // sinh(x + iy) = sinh(x)cos(y) + i*cosh(x)sin(y)
        let sinh_z = vec2<f32>(
            sinh(z.x) * cos(z.y),
            cosh(z.x) * sin(z.y)
        );

        let sinh_sq = complex_mul(sinh_z, sinh_z);
        let sinh_fourth = complex_mul(sinh_sq, sinh_sq);
        let z_next = vec2<f32>(abs(sinh_fourth.x), abs(sinh_fourth.y)) + c;

        if (!complex_is_finite_sinh_julia(z_next)) {
            return i;
        }

        z = z_next;
    }

    return params.max_iter;
}