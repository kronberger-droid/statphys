pub mod analysis;
pub mod d2q9;
pub mod fft;
pub mod fluid;
pub mod free_energy;
pub mod runner;
pub mod types;

pub use fluid::{Fluid2D, FluidParams};
pub use runner::{
    MetastableConfig, RunHistory, RunOutput, SpinodalConfig, run_and_collect, run_snapshots,
    snapshot_to_2d,
};
pub use types::Real;
