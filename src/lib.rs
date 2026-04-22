pub mod lb;
pub mod mc;

use std::fs::File;
use std::io::Write;

use serde::Serialize;

/// Helper function to create file plus its parents (panics on err)
pub fn create_data_file(path: &str) -> File {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).expect("failed to create parent dir")
    }
    std::fs::File::create(path).expect("failed to create output file")
}

/// Serialize `value` as pretty JSON to `path`, creating parent dirs as needed.
pub fn write_json(path: &str, value: &impl Serialize) {
    let mut file = create_data_file(path);
    serde_json::to_writer_pretty(&mut file, value).unwrap();
    writeln!(file).unwrap();
    println!("Wrote {path}");
}

/// Analytical solution for diffusion with reflecting walls on [-L/2, +L/2].
///
/// p(x, τ) = 1 + 2 Σ cos(2πnx) exp(-(2πn)²τ)
///
/// where τ = Dt/L² is the reduced time and L = 1.
pub fn p_analytical_reflecting(x: f64, tau: f64, n_terms: usize) -> f64 {
    let mut result = 1.0;
    for n in 1..=n_terms {
        let k = 2.0 * std::f64::consts::PI * n as f64;
        result += 2.0 * (k * x).cos() * (-k * k * tau).exp();
    }
    result
}

/// Reflect position into [-half_l, +half_l] by folding at walls.
pub fn reflect(mut x: f64, half_l: f64) -> f64 {
    let l = 2.0 * half_l;
    loop {
        if x > half_l {
            x = l - x;
        } else if x < -half_l {
            x = -l - x;
        } else {
            return x;
        }
    }
}

/// Histogram result: bin centers and probability density.
pub struct Histogram {
    pub bin_centers: Vec<f64>,
    pub density: Vec<f64>,
}

/// Bin positions into a histogram with a fixed range [lo, hi].
pub fn histogram_fixed(
    positions: &[f64],
    n_bins: usize,
    lo: f64,
    hi: f64,
) -> Histogram {
    let bin_width = (hi - lo) / n_bins as f64;
    let mut counts = vec![0usize; n_bins];

    for &x in positions {
        let idx = ((x - lo) / bin_width) as usize;
        counts[idx.min(n_bins - 1)] += 1;
    }

    let n = positions.len() as f64;
    Histogram {
        bin_centers: (0..n_bins)
            .map(|i| lo + (i as f64 + 0.5) * bin_width)
            .collect(),
        density: counts.iter().map(|&c| c as f64 / (n * bin_width)).collect(),
    }
}

/// Bin positions into a histogram with auto-detected range (5% margin on each side).
pub fn histogram_auto(positions: &[f64], n_bins: usize) -> Histogram {
    let min = positions.iter().cloned().reduce(f64::min).unwrap();
    let max = positions.iter().cloned().reduce(f64::max).unwrap();
    let margin = (max - min) * 0.05;
    histogram_fixed(positions, n_bins, min - margin, max + margin)
}

/// Serializable curve for analytical solutions.
#[derive(Serialize, Clone)]
pub struct AnalyticalCurve {
    pub dt_over_l2: f64,
    pub x: Vec<f64>,
    pub p: Vec<f64>,
}

/// Compute analytical reflecting-wall curves for a set of reduced times.
pub fn analytical_reflecting_curves(
    reduced_times: &[f64],
    n_points: usize,
    n_terms: usize,
) -> Vec<AnalyticalCurve> {
    reduced_times
        .iter()
        .map(|&tau| {
            let x: Vec<f64> = (0..=n_points)
                .map(|i| -0.5 + i as f64 / n_points as f64)
                .collect();
            let p: Vec<f64> = x
                .iter()
                .map(|&xi| p_analytical_reflecting(xi, tau, n_terms))
                .collect();
            AnalyticalCurve {
                dt_over_l2: tau,
                x,
                p,
            }
        })
        .collect()
}

/// 2D Position convenience struct
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Position2D {
    pub x: f64,
    pub y: f64,
}

impl Position2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// 2D Position convenience struct
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Position3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}
