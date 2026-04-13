use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::Serialize;

use crate::Position2D;

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
    pub positions: Vec<Position2D>,
    pub trajectory: Vec<Vec<Position2D>>,
    pub box_lengths: Vec<f64>,
    pub energies: Vec<f64>,
    pub saved_sweeps: Vec<usize>,
}

pub struct MonteCarloSystem {
    positions: Vec<Position2D>,
    box_length: f64,
    sigma: f64,
    max_displacement: f64,
    max_delta_log_area: f64,
    pressure: Option<f64>,
    ensemble: Ensemble,
    cell_list: CellList,
    rng: StdRng,
}

#[allow(clippy::complexity)]
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
        let rng = StdRng::seed_from_u64(seed);
        let n_side = (n_particles as f64).sqrt().ceil() as usize;
        let spacing = box_length / n_side as f64;

        let mut cell_list = CellList::new(box_length, sigma);
        let mut positions = Vec::new();

        for k in 0..n_particles {
            let i = k / n_side;
            let j = k % n_side;
            let pos = Position2D::new(
                i as f64 * spacing + spacing / 2.0,
                j as f64 * spacing + spacing / 2.0,
            );
            positions.push(pos);
        }

        cell_list.rebuild(&positions);

        MonteCarloSystem {
            positions,
            box_length,
            sigma,
            max_displacement,
            max_delta_log_area,
            pressure,
            ensemble,
            cell_list,
            rng,
        }
    }

    /// Check if placing a particle at `pos` overlaps any disk (ignoring `skip_index`).
    fn has_overlap(&self, pos: Position2D, skip_index: Option<usize>) -> bool {
        for n in self.cell_list.neighbors(pos) {
            if Some(n) == skip_index {
                continue;
            }

            let other = self.positions[n];
            let mut dx = pos.x - other.x;
            let mut dy = pos.y - other.y;
            dx -= self.box_length * (dx / self.box_length).round();
            dy -= self.box_length * (dy / self.box_length).round();

            if dx * dx + dy * dy < self.sigma * self.sigma {
                return true;
            }
        }
        false
    }

    fn attempt_particle_move(&mut self) -> bool {
        let idx = self.rng.random_range(0..self.positions.len()) as usize;
        let old_pos = self.positions[idx];

        let dx = self
            .rng
            .random_range(-self.max_displacement..=self.max_displacement);
        let dy = self
            .rng
            .random_range(-self.max_displacement..=self.max_displacement);

        let new_pos = Position2D::new(
            (old_pos.x + dx).rem_euclid(self.box_length),
            (old_pos.y + dy).rem_euclid(self.box_length),
        );

        if self.has_overlap(new_pos, Some(idx)) {
            false
        } else {
            self.positions[idx] = new_pos;
            self.cell_list.update_particle(idx, old_pos, new_pos);
            true
        }
    }

    fn attempt_volume_move(&mut self) -> bool {
        if self.ensemble != Ensemble::Npt {
            return false;
        }
        let pressure = self.pressure.unwrap();
        let n = self.positions.len() as f64;

        let old_positions = self.positions.clone();
        let old_box_length = self.box_length;
        let old_area = self.box_length * self.box_length;

        // Log-area perturbation
        let delta_log_a = self
            .rng
            .random_range(-self.max_delta_log_area..=self.max_delta_log_area);
        let new_area = old_area * delta_log_a.exp();
        let new_box_length = new_area.sqrt();
        let scale = new_box_length / old_box_length;

        // Rescale all positions
        for pos in &mut self.positions {
            pos.x = (pos.x * scale).rem_euclid(new_box_length);
            pos.y = (pos.y * scale).rem_euclid(new_box_length);
        }
        self.box_length = new_box_length;

        // Check for any overlaps in the rescaled configuration
        let overlap = (0..self.positions.len())
            .any(|i| self.has_overlap(self.positions[i], Some(i)));

        // NPT acceptance criterion for hard disks (beta * delta_e = 0 if no overlap)
        let accept = if overlap {
            false
        } else {
            let log_acc = -pressure * (new_area - old_area)
                + n * (new_area / old_area).ln();
            log_acc >= 0.0 || self.rng.random::<f64>().ln() < log_acc
        };

        if accept {
            // Rebuild cell list for new box geometry
            self.cell_list = CellList::new(self.box_length, self.sigma);
            self.cell_list.rebuild(&self.positions);
            true
        } else {
            self.positions = old_positions;
            self.box_length = old_box_length;
            false
        }
    }

    pub fn run(&mut self, t_end: usize, save_every: usize) -> SimulationResult {
        let n = self.positions.len();

        let mut trajectory = Vec::new();
        let mut box_lengths = Vec::new();
        let mut energies = Vec::new();
        let mut saved_sweeps = Vec::new();

        let mut n_move_attempts: usize = 0;
        let mut n_move_accepts: usize = 0;
        let mut n_volume_attempts: usize = 0;
        let mut n_volume_accepts: usize = 0;

        // Save initial state
        trajectory.push(self.positions.clone());
        box_lengths.push(self.box_length);
        energies.push(0.0); // Hard disks: energy is always 0 if no overlaps
        saved_sweeps.push(0);

        for sweep in 1..=t_end {
            // N particle move attempts per sweep
            for _ in 0..n {
                n_move_attempts += 1;
                if self.attempt_particle_move() {
                    n_move_accepts += 1;
                }
            }

            // One volume move attempt per sweep (NPT only)
            if self.ensemble == Ensemble::Npt {
                n_volume_attempts += 1;
                if self.attempt_volume_move() {
                    n_volume_accepts += 1;
                }
            }

            if sweep % save_every == 0 {
                trajectory.push(self.positions.clone());
                box_lengths.push(self.box_length);
                energies.push(0.0);
                saved_sweeps.push(sweep);
            }
        }

        SimulationResult {
            move_acceptance: n_move_accepts as f64
                / n_move_attempts.max(1) as f64,
            volume_acceptance: n_volume_accepts as f64
                / n_volume_attempts.max(1) as f64,
            positions: self.positions.clone(),
            trajectory,
            box_lengths,
            energies,
            saved_sweeps,
        }
    }
}
