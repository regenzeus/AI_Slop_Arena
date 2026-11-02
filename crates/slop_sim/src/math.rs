//! Transcendental functions routed through `libm`.
//!
//! `f32::powf`, `ln`, `sin` and friends call the platform's C library, whose
//! results can differ in the last bit between Windows, macOS and Linux. That
//! would break shared seeds, so all simulation code uses these wrappers.

pub fn powf(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}

pub fn ln(x: f32) -> f32 {
    libm::logf(x)
}

pub fn exp(x: f32) -> f32 {
    libm::expf(x)
}

pub fn sin(x: f32) -> f32 {
    libm::sinf(x)
}

pub fn cos(x: f32) -> f32 {
    libm::cosf(x)
}

pub fn atan2(y: f32, x: f32) -> f32 {
    libm::atan2f(y, x)
}

/// IEEE square root is exactly specified, so the std version is safe.
pub fn sqrt(x: f32) -> f32 {
    x.sqrt()
}
