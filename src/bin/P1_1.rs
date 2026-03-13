use ndarray::{Array1, Array2, Axis};
use rand_distr::{Distribution, Normal};
use serde::Serialize;
use statphys::histogram_auto;

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

    let hist_steps = [(2, 1), (5, 4), (20, 19), (100, 99)];
    let histograms: Vec<HistogramData> = hist_steps
        .iter()
        .map(|&(step, row_idx)| {
            let positions = samples.row(row_idx);
            let h = histogram_auto(positions.as_slice().unwrap(), 50);
            HistogramData {
                step,
                bin_centers: h.bin_centers,
                density: h.density,
            }
        })
        .collect();

    let output = Output {
        sigma2,
        n_traj,
        n_steps,
        x2_mean: x2_mean.to_vec(),
        histograms,
    };

    let file = statphys::create_data_file("data/P1_1.json");
    serde_json::to_writer_pretty(file, &output).expect("failed to write JSON");
}
