use serde::Serialize;

#[derive(Serialize)]
struct Output {
    reduced_times: Vec<f64>,
    curves: Vec<Curve>,
}

#[derive(Serialize)]
struct Curve {
    dt_over_l2: f64,
    x: Vec<f64>,
    p: Vec<f64>,
}

/// Analytical solution: p(x,t) = 1/L + 2/L * sum cos(2*pi*n*x/L) * exp(-D*(2*pi*n/L)^2 * t)
/// Using reduced time tau = Dt/L^2 and setting L = 1:
/// p(x, tau) = 1 + 2 * sum cos(2*pi*n*x) * exp(-(2*pi*n)^2 * tau)
fn p_analytical(x: f64, tau: f64, n_terms: usize) -> f64 {
    let mut result = 1.0; // L = 1
    for n in 1..=n_terms {
        let k = 2.0 * std::f64::consts::PI * n as f64;
        result += 2.0 * (k * x).cos() * (-k * k * tau).exp();
    }
    result
}

fn main() {
    let reduced_times = vec![0.001, 0.01, 0.02, 0.03, 0.04, 0.05, 0.1];
    let n_points = 500;
    let n_terms = 200; // truncation of infinite series

    let curves: Vec<Curve> = reduced_times
        .iter()
        .map(|&tau| {
            let x: Vec<f64> = (0..=n_points)
                .map(|i| -0.5 + i as f64 / n_points as f64)
                .collect();
            let p: Vec<f64> = x.iter().map(|&xi| p_analytical(xi, tau, n_terms)).collect();
            Curve { dt_over_l2: tau, x, p }
        })
        .collect();

    let output = Output { reduced_times: reduced_times.clone(), curves };

    let file = std::fs::File::create("data/P1_3.json").expect("failed to create output file");
    serde_json::to_writer_pretty(file, &output).expect("failed to write JSON");
}
