use std::io::Write;

use clap::{Parser, Subcommand};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::Serialize;
use statphys::create_data_file;

#[derive(Parser)]
#[command(about = "Exercise 4.1: Monte Carlo simulations of hard disks")]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

#[derive(Subcommand)]
enum Task {
    /// 1.1a: Simulation time vs particle count (NVT and NPT)
    Timing,
    /// 1.1b: Acceptance rate vs step size at different densities
    Acceptance,
    /// Run all tasks
    All,
}

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

/// Import the hard_disks_mc module, adding its directory to sys.path.
fn import_hsmc<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let sys = py.import("sys")?;
    sys.getattr("path")?
        .call_method1("insert", (0, "exercises/exercise-4/MC"))?;
    py.import("hard_disks_mc")
}

fn task_timing(hsmc: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = hsmc.py();

    let particles = [36, 48, 60, 72, 100];
    let ensembles = ["NVT", "NPT"];
    let density = 0.5;
    let t_end = 200;
    let n_runs = 5;
    let pressure = 5.0; // only used for NPT

    let mut points = Vec::new();

    for &ensemble in &ensembles {
        for &n in &particles {
            let box_length = (n as f64 / density).sqrt();
            let mut times = Vec::new();

            for run in 0..n_runs {
                let kwargs = PyDict::new(py);
                kwargs.set_item("n_particles", n)?;
                kwargs.set_item("t_end", t_end)?;
                kwargs.set_item("box_length", box_length)?;
                kwargs.set_item("ensemble", ensemble)?;
                kwargs.set_item("save_every", t_end)?;
                kwargs.set_item("seed", run)?;
                if matches!(ensemble, "NPT") {
                    kwargs.set_item("pressure", pressure)?;
                };

                let start = std::time::Instant::now();
                hsmc.call_method("run_simulation", (), Some(&kwargs))?;
                let elapsed = start.elapsed().as_secs_f64();

                times.push(elapsed);
            }

            points.push(TimingPoint {
                n_particles: n,
                ensemble: ensemble.to_string(),
                times_s: times,
            });

            println!("  {ensemble} N={n}: done ({n_runs} runs)",);
        }
    }

    let output = TimingOutput {
        t_end,
        n_runs,
        density,
        points,
    };

    let mut file = create_data_file("data/P4_1/timing.json");
    serde_json::to_writer_pretty(&mut file, &output).unwrap();
    writeln!(file).unwrap();
    println!("Wrote data/P4_1/timing.json");

    Ok(())
}

fn task_acceptance(hsmc: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = hsmc.py();

    let n_particles = 36;
    let sigma = 1.0;
    let t_end = 200;
    let displacements = [0.01, 0.1, 0.2, 0.4, 0.5, 0.75, 1.0];
    let densities = [0.1, 0.5, 0.8];

    let mut points = Vec::new();

    for &density in &densities {
        let box_length = (n_particles as f64 / density).sqrt();

        for &max_disp in &displacements {
            let kwargs = PyDict::new(py);
            kwargs.set_item("n_particles", n_particles)?;
            kwargs.set_item("t_end", t_end)?;
            kwargs.set_item("box_length", box_length)?;
            kwargs.set_item("sigma", sigma)?;
            kwargs.set_item("max_displacement", max_disp)?;
            kwargs.set_item("ensemble", "NVT")?;
            kwargs.set_item("save_every", t_end)?;
            kwargs.set_item("seed", 24)?;
            let result =
                hsmc.call_method("run_simulation", (), Some(&kwargs))?;

            let acceptance: f64 =
                result.getattr("move_acceptance")?.extract()?;

            points.push(AcceptancePoint {
                density,
                max_displacement: max_disp,
                acceptance_rate: acceptance,
            });
        }

        println!("  density={density}: done");
    }

    let output = AcceptanceOutput {
        n_particles,
        sigma,
        t_end,
        points,
    };

    let mut file = create_data_file("data/P4_1/acceptance.json");
    serde_json::to_writer_pretty(&mut file, &output).unwrap();
    writeln!(file).unwrap();
    println!("Wrote data/P4_1/acceptance.json");

    Ok(())
}

fn main() -> PyResult<()> {
    let cli = Cli::parse();

    Python::with_gil(|py| {
        let hsmc = import_hsmc(py)?;

        match cli.task {
            Task::Timing => {
                println!("=== Task 1.1a: Timing ===");
                task_timing(&hsmc)?;
            }
            Task::Acceptance => {
                println!("=== Task 1.1b: Acceptance ===");
                task_acceptance(&hsmc)?;
            }
            Task::All => {
                println!("=== Task 1.1a: Timing ===");
                task_timing(&hsmc)?;
                println!("\n=== Task 1.1b: Acceptance ===");
                task_acceptance(&hsmc)?;
            }
        }

        Ok(())
    })
}
