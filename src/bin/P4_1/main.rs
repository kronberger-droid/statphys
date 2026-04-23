#[cfg(feature = "python-backend")]
mod python;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use statphys::mc::system::{Ensemble, MonteCarloSystem};
use statphys::write_json;

#[derive(Parser)]
#[command(about = "Exercise 4.1: Monte Carlo simulations of hard disks")]
struct Cli {
    #[command(subcommand)]
    task: Task,

    /// Simulation backend
    #[arg(long, default_value = "rust", global = true)]
    backend: Backend,
}

#[derive(Clone, Copy, ValueEnum)]
enum Backend {
    Rust,
    #[cfg(feature = "python-backend")]
    Python,
}

#[derive(Subcommand)]
enum Task {
    /// 1.1a: Simulation time vs particle count
    Timing,
    /// 1.1b: Acceptance rate vs step size at different densities
    Acceptance,
    /// 1.2a: Packing fraction vs sweeps (NPT at different pressures)
    Packing,
    /// 1.2b: Henderson equation of state comparison
    Henderson,
    /// Run all tasks
    All,
}

// Shared simulation parameters (used by both backends)
const TIMING_PARTICLES: [usize; 5] = [36, 48, 60, 72, 100];
const TIMING_DENSITY: f64 = 0.5;
const TIMING_T_END: usize = 200;
const TIMING_N_RUNS: usize = 5;
const TIMING_PRESSURE: f64 = 5.0;

const ACCEPTANCE_N_PARTICLES: usize = 36;
const ACCEPTANCE_SIGMA: f64 = 1.0;
const ACCEPTANCE_T_END: usize = 200;
const ACCEPTANCE_DISPLACEMENTS: [f64; 7] =
    [0.01, 0.1, 0.2, 0.4, 0.5, 0.75, 1.0];
const ACCEPTANCE_DENSITIES: [f64; 3] = [0.1, 0.5, 0.8];
const ACCEPTANCE_SEED: u64 = 24;

const PACKING_N_PARTICLES: usize = 64;
const PACKING_SIGMA: f64 = 1.0;
const PACKING_INITIAL_BOX_LENGTH: f64 = 16.0;
const PACKING_PRESSURES: [f64; 3] = [1.0, 2.0, 5.0];
const PACKING_T_END: usize = 2000;
const PACKING_SEED: u64 = 42;

const HENDERSON_PRESSURES: [f64; 11] =
    [0.5, 1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 15.0, 25.0, 50.0, 100.0];
const HENDERSON_EQUILIBRATION: usize = 10000; // sweeps to discard before averaging
const HENDERSON_T_END: usize = 50000;

#[derive(Serialize)]
struct TimingPoint {
    n_particles: usize,
    ensemble: String,
    times_s: Vec<f64>,
}

#[derive(Serialize)]
struct TimingOutput {
    t_end: usize,
    n_runs: usize,
    density: f64,
    points: Vec<TimingPoint>,
}

#[derive(Serialize)]
struct AcceptancePoint {
    density: f64,
    max_displacement: f64,
    acceptance_rate: f64,
}

#[derive(Serialize)]
struct AcceptanceOutput {
    n_particles: usize,
    sigma: f64,
    t_end: usize,
    points: Vec<AcceptancePoint>,
}

#[derive(Serialize)]
struct PackingCurve {
    pressure: f64,
    sweeps: Vec<usize>,
    packing_fractions: Vec<f64>,
}

#[derive(Serialize)]
struct PackingOutput {
    n_particles: usize,
    sigma: f64,
    initial_box_length: f64,
    curves: Vec<PackingCurve>,
}

#[derive(Serialize)]
struct HendersonPoint {
    pressure: f64,
    packing_fraction: f64,
    packing_std: f64,
}

#[derive(Serialize)]
struct HendersonOutput {
    n_particles: usize,
    sigma: f64,
    equilibration_sweeps: usize,
    points: Vec<HendersonPoint>,
}

fn timing_rust() {
    let ensembles = [("NVT", Ensemble::Nvt), ("NPT", Ensemble::Npt)];

    let mut points = Vec::new();

    for &(name, ensemble) in &ensembles {
        for &n in &TIMING_PARTICLES {
            let box_length = (n as f64 / TIMING_DENSITY).sqrt();
            let mut times = Vec::new();

            for run in 0..TIMING_N_RUNS {
                let mut system = MonteCarloSystem::new(
                    n,
                    box_length,
                    1.0,
                    0.1,
                    ensemble,
                    if ensemble == Ensemble::Npt {
                        Some(TIMING_PRESSURE)
                    } else {
                        None
                    },
                    0.02,
                    run as u64,
                );

                let start = std::time::Instant::now();
                system.run(TIMING_T_END, TIMING_T_END);
                let elapsed = start.elapsed().as_secs_f64();

                times.push(elapsed);
            }

            points.push(TimingPoint {
                n_particles: n,
                ensemble: name.to_string(),
                times_s: times,
            });

            println!("  {name} N={n}: done ({} runs)", TIMING_N_RUNS);
        }
    }

    let output = TimingOutput {
        t_end: TIMING_T_END,
        n_runs: TIMING_N_RUNS,
        density: TIMING_DENSITY,
        points,
    };

    write_json("exercises/exercise-4/data/P4_1/timing_rust.json", &output);
}

fn acceptance_rust() {
    let mut points = Vec::new();

    for &density in &ACCEPTANCE_DENSITIES {
        let box_length = (ACCEPTANCE_N_PARTICLES as f64 / density).sqrt();

        for &max_disp in &ACCEPTANCE_DISPLACEMENTS {
            let mut system = MonteCarloSystem::new(
                ACCEPTANCE_N_PARTICLES,
                box_length,
                ACCEPTANCE_SIGMA,
                max_disp,
                Ensemble::Nvt,
                None,
                0.02,
                ACCEPTANCE_SEED,
            );

            let result = system.run(ACCEPTANCE_T_END, ACCEPTANCE_T_END);

            points.push(AcceptancePoint {
                density,
                max_displacement: max_disp,
                acceptance_rate: result.move_acceptance,
            });
        }

        println!("  density={density}: done");
    }

    let output = AcceptanceOutput {
        n_particles: ACCEPTANCE_N_PARTICLES,
        sigma: ACCEPTANCE_SIGMA,
        t_end: ACCEPTANCE_T_END,
        points,
    };

    write_json("exercises/exercise-4/data/P4_1/acceptance_rust.json", &output);
}

fn packing_rust() {
    let v_particle =
        std::f64::consts::PI * (PACKING_SIGMA / 2.0) * (PACKING_SIGMA / 2.0);

    let mut curves = Vec::new();

    for &pressure in &PACKING_PRESSURES {
        let mut system = MonteCarloSystem::new(
            PACKING_N_PARTICLES,
            PACKING_INITIAL_BOX_LENGTH,
            PACKING_SIGMA,
            0.1,
            Ensemble::Npt,
            Some(pressure),
            0.02,
            PACKING_SEED,
        );

        let result = system.run(PACKING_T_END, 1);

        let packing_fractions: Vec<f64> = result
            .box_lengths
            .iter()
            .map(|&l| PACKING_N_PARTICLES as f64 * v_particle / (l * l))
            .collect();

        curves.push(PackingCurve {
            pressure,
            sweeps: result.saved_sweeps,
            packing_fractions,
        });

        println!("  P={pressure}: done");
    }

    let output = PackingOutput {
        n_particles: PACKING_N_PARTICLES,
        sigma: PACKING_SIGMA,
        initial_box_length: PACKING_INITIAL_BOX_LENGTH,
        curves,
    };

    write_json("exercises/exercise-4/data/P4_1/packing_rust.json", &output);
}

fn henderson_rust() {
    let v_particle =
        std::f64::consts::PI * (PACKING_SIGMA / 2.0) * (PACKING_SIGMA / 2.0);

    let mut points = Vec::new();

    for &pressure in &HENDERSON_PRESSURES {
        let mut system = MonteCarloSystem::new(
            PACKING_N_PARTICLES,
            PACKING_INITIAL_BOX_LENGTH,
            PACKING_SIGMA,
            0.1,
            Ensemble::Npt,
            Some(pressure),
            0.02,
            PACKING_SEED,
        );

        let result = system.run(HENDERSON_T_END, 1);

        let packing_fractions: Vec<f64> = result
            .box_lengths
            .iter()
            .map(|&l| PACKING_N_PARTICLES as f64 * v_particle / (l * l))
            .collect();

        let equilibrated = &packing_fractions[HENDERSON_EQUILIBRATION..];
        let n = equilibrated.len();

        let mean_phi: f64 = equilibrated.iter().sum::<f64>() / n as f64;

        let std_phi: f64 = (equilibrated
            .iter()
            .map(|phi| (phi - mean_phi).powi(2))
            .sum::<f64>()
            / n as f64)
            .sqrt();

        points.push(HendersonPoint {
            pressure,
            packing_fraction: mean_phi,
            packing_std: std_phi,
        });

        println!("  P={pressure}: phi={mean_phi:.4} +/- {std_phi:.4}");
    }

    let output = HendersonOutput {
        n_particles: PACKING_N_PARTICLES,
        sigma: PACKING_SIGMA,
        equilibration_sweeps: HENDERSON_EQUILIBRATION,
        points,
    };

    write_json("exercises/exercise-4/data/P4_1/henderson_rust.json", &output);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.backend {
        Backend::Rust => {
            match cli.task {
                Task::Timing => {
                    println!("=== Task 1.1a: Timing (Rust) ===");
                    timing_rust();
                }
                Task::Acceptance => {
                    println!("=== Task 1.1b: Acceptance (Rust) ===");
                    acceptance_rust();
                }
                Task::Packing => {
                    println!("=== Task 1.2a: Packing fraction (Rust) ===");
                    packing_rust();
                }
                Task::Henderson => {
                    println!("=== Task 1.2b: Henderson EOS (Rust) ===");
                    henderson_rust();
                }
                Task::All => {
                    println!("=== Task 1.1a: Timing (Rust) ===");
                    timing_rust();
                    println!("\n=== Task 1.1b: Acceptance (Rust) ===");
                    acceptance_rust();
                    println!("\n=== Task 1.2a: Packing fraction (Rust) ===");
                    packing_rust();
                    println!("\n=== Task 1.2b: Henderson EOS (Rust) ===");
                    henderson_rust();
                }
            }
            Ok(())
        }
        #[cfg(feature = "python-backend")]
        Backend::Python => python::run_tasks(&cli.task).map_err(Into::into),
    }
}
