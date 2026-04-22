//! Library-level runners for single LB simulations. Mirrors the two main entry
//! points of `binary_LB.py`: `make_spinodal_example` + `run_and_collect`, and
//! `make_metastable_example` + `run_and_collect`.

use rand_distr::StandardNormal;
use serde::Serialize;

use crate::lb::free_energy::{binodal_phi, spinodal_phi};
use crate::lb::types::Real;
use crate::lb::{Fluid2D, FluidParams};

/// Phi history in the simulation's native precision plus metadata, returned by
/// `run_snapshots`. Callers that need f64 snapshots derive them frame-by-frame.
pub struct RunOutput<R: Real> {
    pub phi_snapshots: Vec<Vec<R>>,
    pub times: Vec<f64>,
    pub nx: usize,
    pub ny: usize,
}

/// JSON schema matching the history dict returned by `binary_LB.run_and_collect`.
/// Velocity fields are omitted; add them if a downstream plot needs them.
#[derive(Serialize)]
pub struct RunHistory {
    pub times: Vec<f64>,
    pub steps: Vec<usize>,
    pub phi: Vec<Vec<Vec<f64>>>,
    pub phi_mean: Vec<f64>,
    pub params: serde_json::Value,
    #[serde(rename = "Tc")]
    pub tc: f64,
    pub phi_binodal: f64,
    pub phi_spinodal: f64,
}

/// Physics parameters for a spinodal quench. Names match `binary_LB.make_spinodal_example`.
#[derive(Clone, Debug)]
pub struct SpinodalConfig {
    pub nx: usize,
    pub ny: usize,
    pub n0: f64,
    pub lam: f64,
    pub t: f64,
    pub kappa: f64,
    pub m_mobility: f64,
    pub tau: f64,
    pub dt: f64,
    pub spinodal_fraction: f64,
    pub phi_noise: f64,
    pub hydrodynamics: bool,
    pub seed: u64,
}

/// Physics parameters for a metastable (nucleation) quench. Names match
/// `binary_LB.make_metastable_example`.
#[derive(Clone, Debug)]
pub struct MetastableConfig {
    pub nx: usize,
    pub ny: usize,
    pub n0: f64,
    pub lam: f64,
    pub t: f64,
    pub kappa: f64,
    pub m_mobility: f64,
    pub tau: f64,
    pub dt: f64,
    pub fraction_of_binodal: f64,
    pub phi_noise: f64,
    pub kt: f64,
    pub hydrodynamics: bool,
    pub seed: u64,
}

impl SpinodalConfig {
    pub fn to_params<R: Real>(&self) -> FluidParams {
        let t_r = R::from_f64_lossy(self.t);
        let lam_r = R::from_f64_lossy(self.lam);
        let n0_r = R::from_f64_lossy(self.n0);
        let phi_spin = spinodal_phi::<R>(t_r, lam_r, n0_r).to_f64_lossy();
        FluidParams {
            nx: self.nx,
            ny: self.ny,
            n0: self.n0,
            lam: self.lam,
            t: self.t,
            kappa: self.kappa,
            m_mobility: self.m_mobility,
            tau: self.tau,
            dt: self.dt,
            phi0: (2.0 * self.spinodal_fraction - 1.0) * phi_spin,
            phi_noise: self.phi_noise,
            kt: 0.0,
            seed: self.seed,
            hydrodynamics: self.hydrodynamics,
            dealias_clip: 0.999999,
        }
    }

    /// Per-case metadata object that ends up in `RunHistory::params`.
    pub fn metadata(&self, phi0: f64) -> serde_json::Value {
        serde_json::json!({
            "Nx": self.nx, "Ny": self.ny, "n0": self.n0, "lam": self.lam, "T": self.t,
            "kappa": self.kappa, "M": self.m_mobility, "tau": self.tau, "dt": self.dt,
            "phi0": phi0, "phi_noise": self.phi_noise, "kT": 0.0,
            "seed": self.seed, "hydrodynamics": self.hydrodynamics,
            "spinodal_fraction": self.spinodal_fraction,
        })
    }
}

impl MetastableConfig {
    pub fn to_params<R: Real>(&self) -> FluidParams {
        let t_r = R::from_f64_lossy(self.t);
        let lam_r = R::from_f64_lossy(self.lam);
        let n0_r = R::from_f64_lossy(self.n0);
        let phi_bin = binodal_phi::<R>(t_r, lam_r, n0_r).to_f64_lossy();
        FluidParams {
            nx: self.nx,
            ny: self.ny,
            n0: self.n0,
            lam: self.lam,
            t: self.t,
            kappa: self.kappa,
            m_mobility: self.m_mobility,
            tau: self.tau,
            dt: self.dt,
            phi0: (2.0 * self.fraction_of_binodal - 1.0) * phi_bin,
            phi_noise: self.phi_noise,
            kt: self.kt,
            seed: self.seed,
            hydrodynamics: self.hydrodynamics,
            dealias_clip: 0.999999,
        }
    }

    pub fn metadata(&self, phi0: f64) -> serde_json::Value {
        serde_json::json!({
            "Nx": self.nx, "Ny": self.ny, "n0": self.n0, "lam": self.lam, "T": self.t,
            "kappa": self.kappa, "M": self.m_mobility, "tau": self.tau, "dt": self.dt,
            "phi0": phi0, "phi_noise": self.phi_noise, "kT": self.kt,
            "seed": self.seed, "hydrodynamics": self.hydrodynamics,
            "fraction_of_binodal": self.fraction_of_binodal,
        })
    }
}

/// Run a simulation and record phi snapshots in the native precision `R`.
pub fn run_snapshots<R: Real>(
    params: FluidParams,
    steps: usize,
    snapshot_every: usize,
) -> RunOutput<R>
where
    StandardNormal: rand_distr::Distribution<R>,
{
    let nx = params.nx;
    let ny = params.ny;
    let mut sim = Fluid2D::<R>::new(params);

    let mut phi_snapshots: Vec<Vec<R>> = Vec::new();
    let mut times: Vec<f64> = Vec::new();
    let snap = |sim: &Fluid2D<R>, phi: &mut Vec<Vec<R>>, t: &mut Vec<f64>| {
        phi.push(sim.phi_vec().to_vec());
        t.push(sim.time_f64());
    };

    snap(&sim, &mut phi_snapshots, &mut times);
    let nblocks = steps / snapshot_every;
    let remainder = steps % snapshot_every;
    for _ in 0..nblocks {
        sim.step(snapshot_every);
        snap(&sim, &mut phi_snapshots, &mut times);
    }
    if remainder > 0 {
        sim.step(remainder);
        snap(&sim, &mut phi_snapshots, &mut times);
    }
    RunOutput { phi_snapshots, times, nx, ny }
}

/// Flat row-major phi → 2D `Vec<Vec<f64>>`.
pub fn snapshot_to_2d<R: Real>(phi: &[R], nx: usize, ny: usize) -> Vec<Vec<f64>> {
    (0..ny)
        .map(|iy| phi[iy * nx..(iy + 1) * nx].iter().map(|v| v.to_f64_lossy()).collect())
        .collect()
}

/// Run a parameter set and build a `RunHistory` with `extras` merged into `params`.
pub fn run_and_collect<R: Real>(
    params: FluidParams,
    steps: usize,
    snapshot_every: usize,
    extras: serde_json::Value,
) -> RunHistory
where
    StandardNormal: rand_distr::Distribution<R>,
{
    let t_r = R::from_f64_lossy(params.t);
    let lam_r = R::from_f64_lossy(params.lam);
    let n0_r = R::from_f64_lossy(params.n0);
    let tc = 0.5 * params.lam;
    let phi_binodal = binodal_phi::<R>(t_r, lam_r, n0_r).to_f64_lossy();
    let phi_spinodal = spinodal_phi::<R>(t_r, lam_r, n0_r).to_f64_lossy();

    let nx = params.nx;
    let ny = params.ny;
    let out = run_snapshots::<R>(params, steps, snapshot_every);
    let phi_2d: Vec<Vec<Vec<f64>>> = out
        .phi_snapshots
        .iter()
        .map(|flat| snapshot_to_2d::<R>(flat, nx, ny))
        .collect();
    let phi_mean: Vec<f64> = phi_2d
        .iter()
        .map(|frame| {
            let mut s = 0.0;
            let mut n = 0usize;
            for row in frame {
                for &v in row {
                    s += v;
                    n += 1;
                }
            }
            s / n as f64
        })
        .collect();
    let last_t = out.times.last().copied().unwrap_or(1.0);
    let steps_vec: Vec<usize> = out
        .times
        .iter()
        .map(|&t| (t / last_t * steps as f64).round() as usize)
        .collect();

    RunHistory {
        times: out.times,
        steps: steps_vec,
        phi: phi_2d,
        phi_mean,
        params: extras,
        tc,
        phi_binodal,
        phi_spinodal,
    }
}
