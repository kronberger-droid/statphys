use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize)]
struct Output {
    n_traj: usize,
    steps: Vec<usize>,
    histograms: Vec<HistogramData>,
}

#[derive(Serialize)]
struct HistogramData {
    n_steps: usize,
    counts: HashMap<i64, usize>,
    density: HashMap<i64, f64>,
}

fn discrete_walk_histogram(
    n_steps: usize,
    n_traj: usize,
    rng: &mut impl rand::Rng,
) -> HistogramData {
    // Collect final positions from many independent walks
    let mut final_positions = Vec::with_capacity(n_traj);

    for _ in 0..n_traj {
        let mut x: i64 = 0;
        for _ in 0..n_steps {
            if rng.random_bool(0.5) {
                x += 1;
            } else {
                x -= 1;
            }
        }
        final_positions.push(x);
    }

    let mut counts: HashMap<i64, usize> = HashMap::new();
    for &pos in &final_positions {
        *counts.entry(pos).or_insert(0) += 1;
    }

    // Probability density: since positions are discrete with spacing 2,
    // normalize so that sum over all positions equal 1
    // factor 2 comes from bin width being 2
    let density: HashMap<i64, f64> = counts
        .iter()
        .map(|(&pos, &c)| (pos, c as f64 / (2.0 * n_traj as f64)))
        .collect();

    HistogramData {
        n_steps,
        counts,
        density,
    }
}

fn main() {
    let mut rng = rand::rng();

    let n_traj = 100_000;
    let steps = vec![10, 21];

    let histograms: Vec<HistogramData> = steps
        .iter()
        .map(|&n| discrete_walk_histogram(n, n_traj, &mut rng))
        .collect();

    let output = Output {
        n_traj,
        steps: steps.clone(),
        histograms,
    };

    let file = std::fs::File::create("data/P1_2.json").expect("failed to create output file");
    serde_json::to_writer_pretty(file, &output).expect("failed to write JSON");
}
