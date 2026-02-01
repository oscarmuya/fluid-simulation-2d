use std::f32;

/// Computes the poly6 smoothing kernel value.
///
/// # Arguments
/// * `r` - Distance between two particles
/// * `h` - Smoothing radius (how far we look for neighbors)
///
/// # Returns
/// Kernel weight value indicating neighbor influence
pub fn poly6_kernel(r: f32, h: f32) -> f32 {
    if r >= h {
        return 0.0;
    }
    let temp = h * h - r * r;
    let factor = 4.0 / (f32::consts::PI * h.powi(8));
    factor * temp.powi(3)
}

pub fn spiky_kernel_gradient(dx: f32, dy: f32, distance: f32, h: f32) -> (f32, f32) {
    if distance >= h || distance < 0.0001 {
        return (0.0, 0.0);
    }
    let scale = -45.0 / (f32::consts::PI * h.powi(5));
    let diff = h - distance;
    let factor = scale * diff * diff / distance;
    (factor * dx, factor * dy)
}
