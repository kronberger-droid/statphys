use rand_distr::{Distribution, Normal};
use serde::Serialize;

const L: f64 = 1.0;
const HALF_L: f64 = L / 2.0;
const N_TRAJ: usize = 100_000;
const N_BINS: usize = 60;
const SIGMA: f64 = 0.01;
const N_TERMS: usize = 200;

#[derive(Serialize)]
struct Output {
    methods: Vec<MethodData>,
    analytical: Vec<Curve>,
}

#[derive(Serialize)]
struct MethodData {
    name: String,
    curves: Vec<HistogramCurve>,
}

#[derive(Serialize)]
struct HistogramCurve {
    dt_over_l2: f64,
    bin_centers: Vec<f64>,
    density: Vec<f64>,
}

#[derive(Serialize)]
struct Curve {
    dt_over_l2: f64,
    x: Vec<f64>,
    p: Vec<f64>,
}

/// Analytical solution from exercise 3 (L=1)
fn p_analytical(x: f64, tau: f64) -> f64 {
    let mut result = 1.0;
    for n in 1..=N_TERMS {
        let k = 2.0 * std::f64::consts::PI * n as f64;
        result += 2.0 * (k * x).cos() * (-k * k * tau).exp();
    }
    result
}

/// Reflect position into [-L/2, +L/2] using folding
fn reflect(mut x: f64) -> f64 {
    // Fold into [-L/2, +L/2] by reflecting at walls
    loop {
        if x > HALF_L {
            x = L - x; // reflect at +L/2
        } else if x < -HALF_L {
            x = -L - x; // reflect at -L/2
        } else {
            return x;
        }
    }
}

enum WallMethod {
    Reflect,
    StopAtWall,
    DontMove,
    Redraw,
}

fn simulate(tau: f64, method: &WallMethod, rng: &mut impl rand::Rng) -> Vec<f64> {
    let n_steps = (2.0 * tau / (SIGMA * SIGMA)).round() as usize;
    let normal = Normal::new(0.0, SIGMA).unwrap();
    let mut positions = Vec::with_capacity(N_TRAJ);

    for _ in 0..N_TRAJ {
        let mut x = 0.0_f64;
        for _ in 0..n_steps {
            match method {
                WallMethod::Reflect => {
                    x = reflect(x + normal.sample(rng));
                }
                WallMethod::StopAtWall => {
                    let x_new = x + normal.sample(rng);
                    x = x_new.clamp(-HALF_L, HALF_L);
                }
                WallMethod::DontMove => {
                    let x_new = x + normal.sample(rng);
                    if (-HALF_L..=HALF_L).contains(&x_new) {
                        x = x_new;
                    }
                    // else: x stays the same
                }
                WallMethod::Redraw => loop {
                    let dx = normal.sample(rng);
                    let x_new = x + dx;
                    if (-HALF_L..=HALF_L).contains(&x_new) {
                        x = x_new;
                        break;
                    }
                },
            }
        }
        positions.push(x);
    }
    positions
}

fn histogram(positions: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let bin_width = L / N_BINS as f64;
    let mut counts = vec![0usize; N_BINS];

    for &x in positions {
        let idx = ((x + HALF_L) / bin_width) as usize;
        let idx = idx.min(N_BINS - 1);
        counts[idx] += 1;
    }

    let n = positions.len() as f64;
    let bin_centers: Vec<f64> = (0..N_BINS)
        .map(|i| -HALF_L + (i as f64 + 0.5) * bin_width)
        .collect();
    let density: Vec<f64> = counts.iter().map(|&c| c as f64 / (n * bin_width)).collect();
    (bin_centers, density)
}

fn main() {
    let mut rng = rand::rng();
    let reduced_times = [0.001, 0.01, 0.02, 0.03, 0.04, 0.05, 0.1];

    let methods_spec: Vec<(&str, WallMethod)> = vec![
        ("reflect", WallMethod::Reflect),
        ("stop_at_wall", WallMethod::StopAtWall),
        ("dont_move", WallMethod::DontMove),
        ("redraw", WallMethod::Redraw),
    ];

    let methods: Vec<MethodData> = methods_spec
        .iter()
        .map(|(name, method)| {
            eprintln!("Simulating method: {}", name);
            let curves: Vec<HistogramCurve> = reduced_times
                .iter()
                .map(|&tau| {
                    let positions = simulate(tau, method, &mut rng);
                    let (bin_centers, density) = histogram(&positions);
                    HistogramCurve {
                        dt_over_l2: tau,
                        bin_centers,
                        density,
                    }
                })
                .collect();
            MethodData {
                name: name.to_string(),
                curves,
            }
        })
        .collect();

    // Analytical curves
    let n_points = 500;
    let analytical: Vec<Curve> = reduced_times
        .iter()
        .map(|&tau| {
            let x: Vec<f64> = (0..=n_points)
                .map(|i| -0.5 + i as f64 / n_points as f64)
                .collect();
            let p: Vec<f64> = x.iter().map(|&xi| p_analytical(xi, tau)).collect();
            Curve {
                dt_over_l2: tau,
                x,
                p,
            }
        })
        .collect();

    let output = Output {
        methods,
        analytical,
    };

    let file = statphys::create_data_file("data/P1_4.json");
    serde_json::to_writer_pretty(file, &output).expect("failed to write JSON");
    eprintln!("Done. Wrote data/P1_4.json");
}
