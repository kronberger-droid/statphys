use serde::Serialize;
use statphys::AnalyticalCurve;

#[derive(Serialize)]
struct Output {
    reduced_times: Vec<f64>,
    curves: Vec<AnalyticalCurve>,
}

fn main() {
    let reduced_times = vec![0.001, 0.01, 0.02, 0.03, 0.04, 0.05, 0.1];
    let curves = statphys::analytical_reflecting_curves(&reduced_times, 500, 200);

    let output = Output {
        reduced_times,
        curves,
    };

    let file = statphys::create_data_file("exercises/exercise-1/data/P1_3.json");
    serde_json::to_writer_pretty(file, &output).expect("failed to write JSON");
}
