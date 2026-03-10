use ndarray::{Array1, Array2, Axis};
use rand_distr::{Distribution, Normal};
use serde::Serialize;

#[derive(Serialize)]
struct Output {
    sigma2: f64,
    n_traj: usize,
    n_steps: usize,
    x2_mean: Vec<f64>,
    histograms: Vec<HistogramData>,
}

#[derive(Serialize)]
struct HistogramData {
    step: usize,
    bin_centers: Vec<f64>,
    density: Vec<f64>,
}

fn bin_positions(positions: &[f64], n_bins: usize) -> HistogramData {
    let min = positions.iter().cloned().reduce(f64::min).unwrap();
    let max = positions.iter().cloned().reduce(f64::max).unwrap();
    let margin = (max - min) * 0.05;
    let lo = min - margin;
    let hi = max + margin;
    let bin_width = (hi - lo) / n_bins as f64;

    let mut counts = vec![0usize; n_bins];
    for &x in positions {
        let idx = ((x - lo) / bin_width) as usize;
        let idx = idx.min(n_bins - 1);
        counts[idx] += 1;
    }

    let n = positions.len() as f64;
    let bin_centers: Vec<f64> = (0..n_bins).map(|i| lo + (i as f64 + 0.5) * bin_width).collect();
    let density: Vec<f64> = counts.iter().map(|&c| c as f64 / (n * bin_width)).collect();

    HistogramData {
        step: 0, // filled in by caller
        bin_centers,
        density,
    }
}

fn main() {
    let mut rng = rand::rng();

    let sigma2 = 1.0_f64;
    let normal = Normal::new(0.0, sigma2.sqrt()).unwrap();

    let n_steps = 100;
    let n_traj = 10_000;

    let mut samples = Array2::<f64>::zeros((n_steps, n_traj));

    let mut buf = Array1::<f64>::zeros(n_traj);

    for mut col in samples.columns_mut() {
        col.mapv_inplace(|_| normal.sample(&mut rng));
    }

    for mut row in samples.rows_mut() {
        buf += &row;
        row.assign(&buf);
    }

    let x2_mean = samples.mapv(|x| x * x).mean_axis(Axis(1)).unwrap();

    let hist_steps = [(2, 1), (5, 4), (20, 19), (100, 99)]; // (step, row_index)
    let histograms: Vec<HistogramData> = hist_steps
        .iter()
        .map(|&(step, row_idx)| {
            let positions = samples.row(row_idx);
            let mut h = bin_positions(positions.as_slice().unwrap(), 50);
            h.step = step;
            h
        })
        .collect();

    let output = Output {
        sigma2,
        n_traj,
        n_steps,
        x2_mean: x2_mean.to_vec(),
        histograms,
    };

    let file = std::fs::File::create("data/P1_1.json").expect("failed to create output file");
    serde_json::to_writer_pretty(file, &output).expect("failed to write JSON");
}
