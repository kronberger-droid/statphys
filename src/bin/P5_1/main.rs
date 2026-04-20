mod python;

use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Parser)]
#[command(about = "Exercise 5.1: LB simulation of binary fluids")]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

#[derive(Subcommand)]
pub enum Task {
    /// 1a: Symmetric run, temperature sweep (save final phi snapshots)
    Temperatures,
    /// 1b: Fixed phase point, timestep sweep (stability study)
    Timesteps,
    /// 1c: Asymmetric runs at different spinodal fractions
    Asymmetric,
    /// 2a: Domain-growth L(t) for two viscosities
    DomainGrowth,
    /// 3a: Spinodal vs. nucleation, largest-cluster time series
    Nucleation,
    /// 3b: Minority-cell count for the same three runs
    MinorityCount,
    /// Run everything
    All,
}

// --- Shared parameters (task sheet constants) -----------------------------

// Task 1a
pub const TASK1_LAM: f64 = 1.1;
pub const TASK1_TEMPERATURES: [f64; 3] = [0.55, 0.45, 0.3];
pub const TASK1_STEPS: usize = 20000;
pub const TASK1_DT: f64 = 0.5;
pub const TASK1_SPINODAL_FRACTION: f64 = 0.5;

// Task 1b (pick one phase point from 1a as the reference)
pub const TASK1B_T: f64 = 0.45;
pub const TASK1B_DT: [f64; 5] = [0.01, 0.3, 1.0, 2.0, 10.0];
pub const TASK1B_STEPS: usize = 20000;

// Task 1c
pub const TASK1C_T: f64 = 0.4;
pub const TASK1C_FRACTIONS: [f64; 3] = [0.4, 0.2, 0.1];
pub const TASK1C_STEPS: usize = 20000;

// Task 2a
pub const TASK2_T: f64 = 0.51;
pub const TASK2_M: f64 = 0.08;
pub const TASK2_DT: f64 = 0.5;
pub const TASK2_STEPS: usize = 100000;
pub const TASK2_TAU: [f64; 2] = [0.7, 5.0];

// Task 3 (nucleation experiment)
pub const TASK3_KT: f64 = 0.004;
pub const TASK3_FRACTION_BINODAL: f64 = 0.2;
pub const TASK3_KAPPA: f64 = 0.2;
pub const TASK3_TAU: f64 = 0.9;
pub const TASK3_DT: f64 = 0.5;
pub const TASK3_M: f64 = 0.2;
pub const TASK3_STEPS: usize = 200000;
/// (T, seed). Seeds are drawn at runtime from entropy so runs 1 and 2
/// at T=0.525 explore different fluctuation histories of the metastable state.
pub const TASK3_TEMPERATURES: [f64; 3] = [0.525, 0.525, 0.45];

pub const SNAPSHOT_EVERY: usize = 500;

// --- Output structs -------------------------------------------------------

#[derive(Serialize)]
pub struct SnapshotOutput {
    pub label: String,
    pub phi_final: Vec<Vec<f64>>,
    pub params: serde_json::Value,
}

#[derive(Serialize)]
pub struct SnapshotCollection {
    pub snapshots: Vec<SnapshotOutput>,
}

#[derive(Serialize)]
pub struct DomainGrowthCurve {
    pub tau: f64,
    pub times: Vec<f64>,
    pub l_of_t: Vec<f64>,
}

#[derive(Serialize)]
pub struct DomainGrowthOutput {
    pub curves: Vec<DomainGrowthCurve>,
}

#[derive(Serialize)]
pub struct NucleationCurve {
    pub label: String,
    pub temperature: f64,
    pub threshold: f64,
    pub seed: u64,
    pub times: Vec<f64>,
    pub largest_cluster: Vec<i64>,
    pub minority_count: Vec<i64>,
}

#[derive(Serialize)]
pub struct NucleationOutput {
    pub curves: Vec<NucleationCurve>,
}

// --- Entrypoint -----------------------------------------------------------

fn main() -> pyo3::PyResult<()> {
    let cli = Cli::parse();
    python::run_tasks(&cli.task)
}
