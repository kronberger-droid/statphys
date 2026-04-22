//! Standalone CLI wrapper around the library `statphys::lb`.
//!
//! Mirrors the calling surface of `binary_LB.py::make_spinodal_example` /
//! `make_metastable_example` + `run_and_collect`, so switching from the Python
//! library to this binary only requires rewriting the invocation, not the
//! parameter set.

#[cfg(feature = "python-backend")]
mod python;

use clap::{Parser, Subcommand, ValueEnum};
use rand_distr::StandardNormal;
use statphys::lb::types::Real;
use statphys::lb::{
    Fluid2D, FluidParams, MetastableConfig, SpinodalConfig, run_and_collect,
};
use statphys::write_json;

#[derive(Parser)]
#[command(
    about = "Binary-fluid lattice Boltzmann CLI (Rust port of binary_LB.py).",
    version
)]
struct Cli {
    #[command(subcommand)]
    task: Task,

    #[arg(long, default_value = "rust", global = true)]
    backend: Backend,

    #[arg(long, default_value = "f64", global = true)]
    precision: Precision,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Backend {
    Rust,
    #[cfg(feature = "python-backend")]
    Python,
}

#[derive(Clone, Copy, ValueEnum, PartialEq, Eq, Debug)]
pub enum Precision {
    F32,
    F64,
}

#[derive(Subcommand)]
pub enum Task {
    /// Single run from a spinodal quench.
    Spinodal(SpinodalArgs),
    /// Single run from a metastable initial state.
    Metastable(MetastableArgs),
    /// Micro-benchmark: Nx × Ny × steps, report ms/step.
    Bench {
        #[arg(long, default_value_t = 128)]
        nx: usize,
        #[arg(long, default_value_t = 128)]
        ny: usize,
        #[arg(long, default_value_t = 500)]
        steps: usize,
    },
}

#[derive(clap::Args, Clone)]
pub struct SpinodalArgs {
    #[arg(long, default_value_t = 128)]
    pub nx: usize,
    #[arg(long, default_value_t = 128)]
    pub ny: usize,
    #[arg(long, default_value_t = 1.0)]
    pub n0: f64,
    #[arg(long, default_value_t = 1.1)]
    pub lam: f64,
    /// Temperature (`T` in the Python API).
    #[arg(long, default_value_t = 0.50)]
    pub temperature: f64,
    #[arg(long, default_value_t = 0.12)]
    pub kappa: f64,
    /// Mobility (`M` in the Python API).
    #[arg(long, default_value_t = 0.08)]
    pub mobility: f64,
    #[arg(long, default_value_t = 0.9)]
    pub tau: f64,
    #[arg(long, default_value_t = 0.5)]
    pub dt: f64,
    #[arg(long, default_value_t = 0.40)]
    pub spinodal_fraction: f64,
    #[arg(long, default_value_t = 2e-3)]
    pub phi_noise: f64,
    /// Disable hydrodynamics (pure Cahn-Hilliard). Default: hydrodynamics on.
    #[arg(long, default_value_t = false)]
    pub no_hydrodynamics: bool,
    #[arg(long, default_value_t = 0)]
    pub seed: u64,
    #[arg(long, default_value_t = 20000)]
    pub steps: usize,
    #[arg(long, default_value_t = 20)]
    pub snapshot_every: usize,
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(clap::Args, Clone)]
pub struct MetastableArgs {
    #[arg(long, default_value_t = 128)]
    pub nx: usize,
    #[arg(long, default_value_t = 128)]
    pub ny: usize,
    #[arg(long, default_value_t = 1.0)]
    pub n0: f64,
    #[arg(long, default_value_t = 1.1)]
    pub lam: f64,
    #[arg(long, default_value_t = 0.50)]
    pub temperature: f64,
    #[arg(long, default_value_t = 0.12)]
    pub kappa: f64,
    #[arg(long, default_value_t = 0.08)]
    pub mobility: f64,
    #[arg(long, default_value_t = 0.9)]
    pub tau: f64,
    #[arg(long, default_value_t = 0.5)]
    pub dt: f64,
    #[arg(long, default_value_t = 0.82)]
    pub fraction_of_binodal: f64,
    #[arg(long, default_value_t = 1e-3)]
    pub phi_noise: f64,
    /// Cahn-Hilliard noise amplitude (`kT` in the Python API).
    #[arg(long, default_value_t = 0.0025)]
    pub kt: f64,
    #[arg(long, default_value_t = false)]
    pub no_hydrodynamics: bool,
    #[arg(long, default_value_t = 0)]
    pub seed: u64,
    #[arg(long, default_value_t = 20000)]
    pub steps: usize,
    #[arg(long, default_value_t = 20)]
    pub snapshot_every: usize,
    #[arg(long)]
    pub output: Option<String>,
}

impl SpinodalArgs {
    pub fn to_config(&self) -> SpinodalConfig {
        SpinodalConfig {
            nx: self.nx,
            ny: self.ny,
            n0: self.n0,
            lam: self.lam,
            t: self.temperature,
            kappa: self.kappa,
            m_mobility: self.mobility,
            tau: self.tau,
            dt: self.dt,
            spinodal_fraction: self.spinodal_fraction,
            phi_noise: self.phi_noise,
            hydrodynamics: !self.no_hydrodynamics,
            seed: self.seed,
        }
    }
}

impl MetastableArgs {
    pub fn to_config(&self) -> MetastableConfig {
        MetastableConfig {
            nx: self.nx,
            ny: self.ny,
            n0: self.n0,
            lam: self.lam,
            t: self.temperature,
            kappa: self.kappa,
            m_mobility: self.mobility,
            tau: self.tau,
            dt: self.dt,
            fraction_of_binodal: self.fraction_of_binodal,
            phi_noise: self.phi_noise,
            kt: self.kt,
            hydrodynamics: !self.no_hydrodynamics,
            seed: self.seed,
        }
    }
}

fn precision_label(p: Precision) -> &'static str {
    match p {
        Precision::F32 => "f32",
        Precision::F64 => "f64",
    }
}

fn run_spinodal<R: Real>(args: &SpinodalArgs, precision: Precision)
where
    StandardNormal: rand_distr::Distribution<R>,
{
    let cfg = args.to_config();
    let params = cfg.to_params::<R>();
    let mut extras = cfg.metadata(params.phi0);
    if let Some(m) = extras.as_object_mut() {
        m.insert("precision".into(), precision_label(precision).into());
    }
    let history = run_and_collect::<R>(params, args.steps, args.snapshot_every, extras);
    let out_path = args.output.clone().unwrap_or_else(|| {
        format!("data/binary_lb/spinodal_{}.json", precision_label(precision))
    });
    write_json(&out_path, &history);
}

fn run_metastable<R: Real>(args: &MetastableArgs, precision: Precision)
where
    StandardNormal: rand_distr::Distribution<R>,
{
    let cfg = args.to_config();
    let params = cfg.to_params::<R>();
    let mut extras = cfg.metadata(params.phi0);
    if let Some(m) = extras.as_object_mut() {
        m.insert("precision".into(), precision_label(precision).into());
    }
    let history = run_and_collect::<R>(params, args.steps, args.snapshot_every, extras);
    let out_path = args.output.clone().unwrap_or_else(|| {
        format!("data/binary_lb/metastable_{}.json", precision_label(precision))
    });
    write_json(&out_path, &history);
}

fn run_bench<R: Real>(nx: usize, ny: usize, steps: usize, label: &str)
where
    StandardNormal: rand_distr::Distribution<R>,
{
    let params = FluidParams { nx, ny, ..FluidParams::default() };
    let mut sim = Fluid2D::<R>::new(params);
    sim.step(10);
    let t0 = std::time::Instant::now();
    sim.step(steps);
    let elapsed = t0.elapsed().as_secs_f64();
    println!(
        "  [{label}] {nx}x{ny} × {steps} steps: {elapsed:.3} s ({:.3} ms/step, {:.1} steps/s)",
        elapsed / steps as f64 * 1000.0,
        steps as f64 / elapsed
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.backend {
        Backend::Rust => match &cli.task {
            Task::Spinodal(args) => {
                println!("=== Spinodal (Rust, {:?}) ===", cli.precision);
                match cli.precision {
                    Precision::F32 => run_spinodal::<f32>(args, cli.precision),
                    Precision::F64 => run_spinodal::<f64>(args, cli.precision),
                }
            }
            Task::Metastable(args) => {
                println!("=== Metastable (Rust, {:?}) ===", cli.precision);
                match cli.precision {
                    Precision::F32 => run_metastable::<f32>(args, cli.precision),
                    Precision::F64 => run_metastable::<f64>(args, cli.precision),
                }
            }
            Task::Bench { nx, ny, steps } => {
                println!("=== Bench (Rust) ===");
                match cli.precision {
                    Precision::F32 => run_bench::<f32>(*nx, *ny, *steps, "rust-f32"),
                    Precision::F64 => run_bench::<f64>(*nx, *ny, *steps, "rust-f64"),
                }
            }
        },
        #[cfg(feature = "python-backend")]
        Backend::Python => python::run(&cli.task)?,
    }

    Ok(())
}
