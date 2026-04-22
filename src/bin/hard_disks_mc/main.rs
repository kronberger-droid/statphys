//! Standalone CLI wrapper around `statphys::mc` for hard-disk Monte Carlo.
//!
//! Mirrors the calling surface of `hard_disks_mc.py::run_simulation`, so
//! switching from `import hard_disks_mc` to this binary only requires rewriting
//! the invocation, not the parameter set or output schema.

#[cfg(feature = "python-backend")]
mod python;

use clap::{Parser, ValueEnum};
use serde::Serialize;
use statphys::Position2D;
use statphys::mc::system::{Ensemble, MonteCarloSystem};
use statphys::write_json;

#[derive(Parser)]
#[command(
    about = "2D hard-disk Monte Carlo CLI (Rust port of hard_disks_mc.py).",
    version
)]
struct Cli {
    #[arg(long, default_value_t = 64)]
    n_particles: usize,
    #[arg(long, default_value_t = 100)]
    t_end: usize,
    #[arg(long, default_value_t = 1.0)]
    temperature: f64,
    #[arg(long, default_value_t = 5.0)]
    box_length: f64,
    #[arg(long)]
    pressure: Option<f64>,
    #[arg(long, default_value_t = 1.0)]
    sigma: f64,
    #[arg(long, default_value_t = 1.0)]
    epsilon: f64,
    #[arg(long, default_value_t = 0.1)]
    max_displacement: f64,
    /// `max_delta_log_volume` in the Python API. NPT-only.
    #[arg(long, default_value_t = 0.02)]
    max_delta_log_area: f64,
    #[arg(long, default_value = "nvt")]
    ensemble: EnsembleArg,
    #[arg(long, default_value_t = 10)]
    save_every: usize,
    #[arg(long, default_value_t = 0)]
    seed: u64,
    #[arg(long, default_value = "square")]
    initialization: InitArg,
    #[arg(long)]
    output: Option<String>,

    #[arg(long, default_value = "rust")]
    backend: Backend,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Backend {
    Rust,
    #[cfg(feature = "python-backend")]
    Python,
}

#[derive(Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum EnsembleArg {
    Nvt,
    Npt,
}

#[derive(Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum InitArg {
    Square,
    Random,
}

/// Output schema matching `hard_disks_mc.MonteCarloResult` (positions as `[x,y]`
/// tuples rather than `{x, y}` objects, to match the NumPy-array serialization
/// the Python reference produces).
#[derive(Serialize)]
pub struct SimulationResult {
    pub positions: Vec<[f64; 2]>,
    pub trajectory: Vec<Vec<[f64; 2]>>,
    pub box_lengths: Vec<f64>,
    pub energies: Vec<f64>,
    pub saved_sweeps: Vec<usize>,
    pub move_acceptance: f64,
    pub volume_acceptance: f64,
    pub metadata: serde_json::Value,
}

fn to_pairs(ps: &[Position2D]) -> Vec<[f64; 2]> {
    ps.iter().map(|p| [p.x, p.y]).collect()
}

fn run_rust(cli: &Cli) -> SimulationResult {
    let ensemble = match cli.ensemble {
        EnsembleArg::Nvt => Ensemble::Nvt,
        EnsembleArg::Npt => Ensemble::Npt,
    };
    if cli.ensemble == EnsembleArg::Npt && cli.pressure.is_none() {
        panic!("--pressure is required for --ensemble npt");
    }
    if cli.initialization == InitArg::Random {
        panic!("--initialization random is not yet supported by the Rust backend; use `square`");
    }

    let mut sim = MonteCarloSystem::new(
        cli.n_particles,
        cli.box_length,
        cli.sigma,
        cli.max_displacement,
        ensemble,
        cli.pressure,
        cli.max_delta_log_area,
        cli.seed,
    );
    let result = sim.run(cli.t_end, cli.save_every);

    let final_box = result.box_lengths.last().copied().unwrap_or(cli.box_length);
    let final_area = final_box * final_box;
    let final_density = cli.n_particles as f64 / final_area;

    let metadata = serde_json::json!({
        "n_particles": cli.n_particles,
        "temperature": cli.temperature,
        "pressure": cli.pressure,
        "sigma": cli.sigma,
        "epsilon": cli.epsilon,
        "final_box_length": final_box,
        "final_area": final_area,
        "final_density": final_density,
        "t_end": cli.t_end,
        "save_every": cli.save_every,
        "include_initial": true,
    });

    SimulationResult {
        positions: to_pairs(&result.positions),
        trajectory: result.trajectory.iter().map(|frame| to_pairs(frame)).collect(),
        box_lengths: result.box_lengths,
        energies: result.energies,
        saved_sweeps: result.saved_sweeps,
        move_acceptance: result.move_acceptance,
        volume_acceptance: result.volume_acceptance,
        metadata,
    }
}

fn default_output_path(cli: &Cli) -> String {
    let tag = match cli.backend {
        Backend::Rust => "rust",
        #[cfg(feature = "python-backend")]
        Backend::Python => "python",
    };
    format!("data/hard_disks_mc/run_{tag}.json")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let out_path = cli.output.clone().unwrap_or_else(|| default_output_path(&cli));

    let result = match cli.backend {
        Backend::Rust => {
            println!("=== hard-disks MC (Rust) ===");
            run_rust(&cli)
        }
        #[cfg(feature = "python-backend")]
        Backend::Python => {
            println!("=== hard-disks MC (Python) ===");
            python::run(&cli)?
        }
    };
    write_json(&out_path, &result);
    Ok(())
}
