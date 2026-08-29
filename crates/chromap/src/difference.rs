use crate::{Color, ColorError};

/// Euclidean distance in OKLab.
pub fn oklab_distance(first: Color, second: Color) -> f64 {
    let first = first.to_oklab();
    let second = second.to_oklab();
    ((first.l - second.l).powi(2) + (first.a - second.a).powi(2) + (first.b - second.b).powi(2))
        .sqrt()
}

/// Euclidean distance in gamma-encoded normalized sRGB.
pub fn srgb_distance(first: Color, second: Color) -> f64 {
    ((first.red() - second.red()).powi(2)
        + (first.green() - second.green()).powi(2)
        + (first.blue() - second.blue()).powi(2))
    .sqrt()
}

/// Finds the nearest candidate in OKLab and returns `(index, distance)`.
pub fn nearest_color(target: Color, candidates: &[Color]) -> Result<(usize, f64), ColorError> {
    if candidates.is_empty() {
        return Err(ColorError::EmptyInput("color candidates"));
    }
    let mut best_index = 0;
    let mut best_distance = oklab_distance(target, candidates[0]);
    for (index, candidate) in candidates.iter().copied().enumerate().skip(1) {
        let distance = oklab_distance(target, candidate);
        if distance < best_distance {
            best_index = index;
            best_distance = distance;
        }
    }
    Ok((best_index, best_distance))
}
