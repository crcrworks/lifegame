use crate::{field::Field, Vec2, DIRECTIONS};
use std::hash::Hash;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Cell {
    pub position: Vec2,
}

impl Hash for Cell {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.hash(state);
    }
}

impl Cell {
    pub fn new(position: impl Into<Vec2>) -> Self {
        Cell {
            position: position.into(),
        }
    }

    pub fn is_next_alive(&self, field: &Field) -> bool {
        let lived_cell_num = self.count_around_lived_cell(field);
        let is_currently_alive = field.cells.contains(self);

        if is_currently_alive {
            (2..=3).contains(&lived_cell_num)
        } else {
            lived_cell_num == 3
        }
    }

    fn count_around_lived_cell(&self, field: &Field) -> u32 {
        DIRECTIONS.into_iter().fold(0, |sum, offset| {
            sum + field
                .cells
                .contains(&Cell::new(self.position + offset.into())) as u32
        })
    }
}
