use crate::Position2D;

/// Spatial cell list for O(1) neighbor lookups with periodic boundary conditions.
pub struct CellList {
    /// Number of cells per side (total cells = cells_per_side²).
    pub cells_per_side: usize,
    pub cell_size: f64,
    cells: Vec<Vec<usize>>,
}

impl CellList {
    /// Build a new cell list for a box of side `box_length` with minimum cell size `sigma`.
    pub fn new(box_length: f64, sigma: f64) -> Self {
        let cells_per_side = (box_length / sigma) as usize;
        let cell_size = box_length / cells_per_side as f64;

        let cells = vec![Vec::new(); cells_per_side * cells_per_side];

        CellList {
            cells_per_side,
            cell_size,
            cells,
        }
    }

    /// Rebuild the entire grid from scratch given particle positions.
    pub fn rebuild(&mut self, positions: &[Position2D]) {
        // Clear current cells (.clear() means memory stays allocated)
        self.cells.iter_mut().for_each(Vec::clear);
        for (i, pos) in positions.iter().enumerate() {
            let idx = self.cell_index(*pos);
            self.cells[idx].push(i);
        }
    }

    /// Move a particle from its old cell to its new cell.
    pub fn update_particle(
        &mut self,
        index: usize,
        old_pos: Position2D,
        new_pos: Position2D,
    ) {
        let old_idx = self.cell_index(old_pos);
        let new_idx = self.cell_index(new_pos);
        if old_idx != new_idx {
            let pos = self.cells[old_idx]
                .iter()
                .position(|&p| p == index)
                .unwrap();
            self.cells[old_idx].swap_remove(pos);
            self.cells[new_idx].push(index);
        }
    }

    /// Iterator over all particle indices in the 3×3 neighborhood of `pos`.
    pub fn neighbors(
        &self,
        pos: Position2D,
    ) -> impl Iterator<Item = usize> + '_ {
        let cx = (pos.x / self.cell_size) as usize;
        let cy = (pos.y / self.cell_size) as usize;

        // Collect the 9 cell flat-indices on the stack
        let mut cell_indices = [0usize; 9];
        let mut i = 0;
        for dx in [-1i32, 0, 1] {
            for dy in [-1i32, 0, 1] {
                let nx = (cx + self.cells_per_side + dx as usize)
                    % self.cells_per_side;
                let ny = (cy + self.cells_per_side + dy as usize)
                    % self.cells_per_side;
                cell_indices[i] = nx * self.cells_per_side + ny;
                i += 1;
            }
        }

        cell_indices
            .into_iter()
            .flat_map(move |idx| self.cells[idx].iter().copied())
    }

    fn cell_index(&self, pos: Position2D) -> usize {
        let cx = (pos.x / self.cell_size) as usize;
        let cy = (pos.y / self.cell_size) as usize;
        cx * self.cells_per_side + cy
    }
}
