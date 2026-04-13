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
        let cell_size = box_length / (box_length / sigma);

        let cells_per_side = (box_length / cell_size) as usize;

        let cells = vec![Vec::new(); cells_per_side * cells_per_side];

        CellList {
            cells_per_side,
            cell_size,
            cells,
        }
    }

    /// Rebuild the entire grid from scratch given particle positions.
    pub fn rebuild(&mut self, positions: &[Position2D], box_length: f64) {
        // Clear current cells (.clear() means memory stays allocated)
        self.cells.iter_mut().for_each(Vec::clear);
        for (i, pos) in positions.iter().enumerate() {
            let cx = box_length / pos.x;
        }
    }

    /// Move a particle from its old cell to its new cell.
    pub fn update_particle(
        &mut self,
        index: usize,
        old_pos: Position2D,
        new_pos: Position2D,
    ) {
        todo!()
    }

    /// Iterator over all particle indices in the 3×3 neighborhood of `pos`.
    pub fn neighbors(
        &self,
        pos: Position2D,
    ) -> impl Iterator<Item = usize> + '_ {
        todo!();
        std::iter::empty()
    }

    fn cell_index(&self, pos: Position2D) -> usize {
        todo!()
    }
}
