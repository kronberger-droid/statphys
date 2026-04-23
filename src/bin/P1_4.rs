use rand_distr::{Distribution, Normal};
use serde::Serialize;
use statphys::AnalyticalCurve;

const L: f64 = 1.0;
const HALF_L: f64 = L / 2.0;
const N_TRAJ: usize = 100_000;
const N_BINS: usize = 60;

// Small sigma for 4a (fine resolution, matches analytical well at early times)
const SIGMA_A: f64 = 0.01;
// Larger sigma for 4b (exaggerates wall-handling artifacts)
const SIGMA_B: f64 = 0.05;

#[derive(Serialize)]
struct Output {
    methods: Vec<MethodData>,
    analytical: Vec<AnalyticalCurve>,
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

enum WallMethod {
    Reflect,
    StopAtWall,
    DontMove,
    Redraw,
}

fn simulate(tau: f64, sigma: f64, method: &WallMethod, rng: &mut impl rand::Rng) -> Vec<f64> {
    let n_steps = (2.0 * tau / (sigma * sigma)).round() as usize;
    let normal = Normal::new(0.0, sigma).unwrap();
    let mut positions = Vec::with_capacity(N_TRAJ);

    for _ in 0..N_TRAJ {
        let mut x = 0.0_f64;
        for _ in 0..n_steps {
            match method {
                WallMethod::Reflect => {
                    x = statphys::reflect(x + normal.sample(rng), HALF_L);
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

fn simulate_method(
    name: &str,
    method: &WallMethod,
    sigma: f64,
    reduced_times: &[f64],
    rng: &mut impl rand::Rng,
) -> MethodData {
    eprintln!("Simulating method: {}", name);
    let curves = reduced_times
        .iter()
        .map(|&tau| {
            let positions = simulate(tau, sigma, method, rng);
            let h = statphys::histogram_fixed(&positions, N_BINS, -HALF_L, HALF_L);
            HistogramCurve {
                dt_over_l2: tau,
                bin_centers: h.bin_centers,
                density: h.density,
            }
        })
        .collect();
    MethodData {
        name: name.to_string(),
        curves,
    }
}

fn main() {
    let mut rng = rand::rng();
    let reduced_times = [0.001, 0.01, 0.02, 0.03, 0.04, 0.05, 0.1];
    let analytical = statphys::analytical_reflecting_curves(&reduced_times, 500, 200);

    // 4a: reflect only, small sigma
    {
        eprintln!("=== P1_4a (sigma={}) ===", SIGMA_A);
        let methods = vec![simulate_method(
            "reflect",
            &WallMethod::Reflect,
            SIGMA_A,
            &reduced_times,
            &mut rng,
        )];

        let output = Output {
            methods,
            analytical: analytical.clone(),
        };
        let file = statphys::create_data_file("exercises/exercise-1/data/P1_4a.json");
        serde_json::to_writer_pretty(file, &output).expect("failed to write JSON");
        eprintln!("Wrote exercises/exercise-1/data/P1_4a.json");
    }

    // 4b: all methods, larger sigma to show wall artifacts
    {
        eprintln!("=== P1_4b (sigma={}) ===", SIGMA_B);
        let methods_spec: Vec<(&str, WallMethod)> = vec![
            ("reflect", WallMethod::Reflect),
            ("stop_at_wall", WallMethod::StopAtWall),
            ("dont_move", WallMethod::DontMove),
            ("redraw", WallMethod::Redraw),
        ];

        let methods: Vec<MethodData> = methods_spec
            .iter()
            .map(|(name, method)| {
                simulate_method(name, method, SIGMA_B, &reduced_times, &mut rng)
            })
            .collect();

        let output = Output {
            methods,
            analytical: analytical.clone(),
        };
        let file = statphys::create_data_file("exercises/exercise-1/data/P1_4b.json");
        serde_json::to_writer_pretty(file, &output).expect("failed to write JSON");
        eprintln!("Wrote exercises/exercise-1/data/P1_4b.json");
    }
}
