use std::collections::HashMap;

use crate::color::EPSILON;
use crate::{Color, ColorError, LinearRgb, Oklab, Rgba8};

/// A representative color and its sample share.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Swatch {
    /// Representative color.
    pub color: Color,
    /// Assigned item count.
    pub population: usize,
    /// Population divided by visible input population.
    pub weight: f64,
}

/// Computes an alpha-aware average in linear-light sRGB.
pub fn average_color(colors: &[Color]) -> Result<Color, ColorError> {
    if colors.is_empty() {
        return Err(ColorError::EmptyInput("colors"));
    }
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    let mut alpha_sum = 0.0;
    for color in colors.iter().copied() {
        let linear = color.to_linear_rgb();
        r += linear.r * color.alpha();
        g += linear.g * color.alpha();
        b += linear.b * color.alpha();
        alpha_sum += color.alpha();
    }
    if alpha_sum <= EPSILON {
        return Ok(Color::TRANSPARENT);
    }
    Color::from_linear_rgb(
        LinearRgb {
            r: r / alpha_sum,
            g: g / alpha_sum,
            b: b / alpha_sum,
        },
        alpha_sum / colors.len() as f64,
    )
}

/// Extracts deterministic representative colors with weighted OKLab k-means.
///
/// Inputs are grouped by rounded RGBA8; fully transparent pixels are ignored.
pub fn dominant_colors(
    colors: &[Color],
    count: usize,
    max_iterations: usize,
) -> Result<Vec<Swatch>, ColorError> {
    if colors.is_empty() {
        return Err(ColorError::EmptyInput("colors"));
    }
    if count == 0 {
        return Err(ColorError::InvalidCount {
            name: "dominant color count",
            value: count,
            minimum: 1,
        });
    }
    if max_iterations == 0 {
        return Err(ColorError::InvalidCount {
            name: "k-means iterations",
            value: max_iterations,
            minimum: 1,
        });
    }

    let mut histogram = HashMap::<Rgba8, usize>::new();
    for color in colors.iter().copied().filter(|c| !c.is_transparent()) {
        let rgba = color.to_rgba8();
        if rgba.a > 0 {
            *histogram.entry(rgba).or_insert(0) += 1;
        }
    }
    if histogram.is_empty() {
        return Ok(vec![Swatch {
            color: Color::TRANSPARENT,
            population: colors.len(),
            weight: 1.0,
        }]);
    }

    let mut points = histogram
        .into_iter()
        .map(|(rgba, population)| {
            let color = Color::from(rgba);
            Point {
                lab: color.to_oklab(),
                alpha: color.alpha(),
                population,
                sample_weight: population as f64 * color.alpha(),
            }
        })
        .collect::<Vec<_>>();
    points.sort_by_key(|point| std::cmp::Reverse(point.population));

    let cluster_count = count.min(points.len());
    if cluster_count == points.len() {
        let total = points.iter().map(|p| p.population).sum::<usize>();
        return points
            .into_iter()
            .map(|point| {
                Ok(Swatch {
                    color: Color::from_oklab_mapped(point.lab, point.alpha)?,
                    population: point.population,
                    weight: point.population as f64 / total as f64,
                })
            })
            .collect();
    }

    let mut centroids = initialize_centroids(&points, cluster_count);
    let mut assignments = vec![0_usize; points.len()];
    for _ in 0..max_iterations {
        for (assignment, point) in assignments.iter_mut().zip(&points) {
            *assignment = nearest_centroid(point.lab, &centroids);
        }
        let mut sums = vec![Accumulator::default(); cluster_count];
        for (point, assignment) in points.iter().zip(assignments.iter().copied()) {
            sums[assignment].add(point);
        }
        let mut movement = 0.0_f64;
        for index in 0..cluster_count {
            let next = if sums[index].weight > EPSILON {
                sums[index].centroid()
            } else {
                farthest_point(&points, &centroids)
            };
            movement = movement.max(lab_distance(centroids[index], next));
            centroids[index] = next;
        }
        if movement < 1.0e-7 {
            break;
        }
    }

    for (assignment, point) in assignments.iter_mut().zip(&points) {
        *assignment = nearest_centroid(point.lab, &centroids);
    }
    let mut sums = vec![Accumulator::default(); cluster_count];
    for (point, assignment) in points.iter().zip(assignments.iter().copied()) {
        sums[assignment].add(point);
    }
    let visible = points.iter().map(|p| p.population).sum::<usize>();
    let mut result = Vec::with_capacity(cluster_count);
    for sum in sums {
        if sum.population == 0 {
            continue;
        }
        result.push(Swatch {
            color: Color::from_oklab_mapped(sum.centroid(), sum.alpha_sum / sum.population as f64)?,
            population: sum.population,
            weight: sum.population as f64 / visible as f64,
        });
    }
    result.sort_by_key(|swatch| std::cmp::Reverse(swatch.population));
    Ok(result)
}

#[derive(Clone, Copy, Debug)]
struct Point {
    lab: Oklab,
    alpha: f64,
    population: usize,
    sample_weight: f64,
}

#[derive(Clone, Copy, Debug, Default)]
struct Accumulator {
    l: f64,
    a: f64,
    b: f64,
    alpha_sum: f64,
    weight: f64,
    population: usize,
}

impl Accumulator {
    fn add(&mut self, point: &Point) {
        self.l += point.lab.l * point.sample_weight;
        self.a += point.lab.a * point.sample_weight;
        self.b += point.lab.b * point.sample_weight;
        self.alpha_sum += point.alpha * point.population as f64;
        self.weight += point.sample_weight;
        self.population += point.population;
    }

    fn centroid(self) -> Oklab {
        Oklab {
            l: self.l / self.weight,
            a: self.a / self.weight,
            b: self.b / self.weight,
        }
    }
}

fn initialize_centroids(points: &[Point], count: usize) -> Vec<Oklab> {
    let mut average = Accumulator::default();
    for point in points {
        average.add(point);
    }
    let mut result = vec![average.centroid()];
    while result.len() < count {
        result.push(farthest_point(points, &result));
    }
    result
}

fn farthest_point(points: &[Point], centroids: &[Oklab]) -> Oklab {
    let mut best = points[0].lab;
    let mut best_score = -1.0_f64;
    for point in points {
        let nearest = centroids
            .iter()
            .copied()
            .map(|centroid| lab_distance_squared(point.lab, centroid))
            .fold(f64::INFINITY, f64::min);
        let score = nearest * point.sample_weight;
        if score > best_score {
            best = point.lab;
            best_score = score;
        }
    }
    best
}

fn nearest_centroid(point: Oklab, centroids: &[Oklab]) -> usize {
    let mut best_index = 0;
    let mut best_distance = lab_distance_squared(point, centroids[0]);
    for (index, centroid) in centroids.iter().copied().enumerate().skip(1) {
        let distance = lab_distance_squared(point, centroid);
        if distance < best_distance {
            best_index = index;
            best_distance = distance;
        }
    }
    best_index
}

fn lab_distance(first: Oklab, second: Oklab) -> f64 {
    lab_distance_squared(first, second).sqrt()
}

fn lab_distance_squared(first: Oklab, second: Oklab) -> f64 {
    (first.l - second.l).powi(2) + (first.a - second.a).powi(2) + (first.b - second.b).powi(2)
}
