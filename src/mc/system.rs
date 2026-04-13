use rand::rngs::StdRng;
use serde::Serialize;

use super::cell_list::CellList;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ensemble {
    Nvt,
    Npt,
}

#[derive(Serialize)]
pub struct SimulationResult {
    pub move_acceptance: f64,
    pub volume_acceptance: f64,
    pub positions: Vec<[f64; 2]>,
    pub trajectory: Vec<Vec<[f64; 2]>>,
    pub box_lengths: Vec<f64>,
    pub energies: Vec<f64>,
    pub saved_sweeps: Vec<usize>,
}

pub struct MonteCarloSystem {
    positions: Vec<[f64; 2]>,
    box_length: f64,
    sigma: f64,
    max_displacement: f64,
    max_delta_log_area: f64,
    pressure: Option<f64>,
    ensemble: Ensemble,
    cell_list: CellList,
    rng: StdRng,
}

impl MonteCarloSystem {
    pub fn new(
        n_particles: usize,
        box_length: f64,
        sigma: f64,
        max_displacement: f64,
        ensemble: Ensemble,
        pressure: Option<f64>,
        max_delta_log_area: f64,
        seed: u64,
    ) -> Self {
        todo!()
    }

    /// Check if placing a particle at `pos` overlaps any disk (ignoring `skip_index`).
    fn has_overlap(&self, pos: [f64; 2], skip_index: Option<usize>) -> bool {
        todo!()
    }

    fn attempt_particle_move(&mut self) -> bool {
        todo!()
    }

    fn attempt_volume_move(&mut self) -> bool {
        todo!()
    }

    pub fn run(&mut self, t_end: usize, save_every: usize) -> SimulationResult {
        todo!()
    }
}
