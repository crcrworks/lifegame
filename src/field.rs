use crate::{cell::Cell, Vec2, DIRECTIONS};
use crossterm::{
    cursor, execute,
    style::{Color, Print, Stylize},
};
use eyre::Result;
use std::{
    collections::HashSet,
    io::{stdout, Stdout},
};

pub struct Field {
    pub stdout: Stdout,
    pub size: Vec2,
    pub cells: HashSet<Cell>,
    pub backuped_cells: Option<HashSet<Cell>>,
    pub is_auto_play_enabled: bool,
    pub step_count: u32,
}

impl Field {
    pub fn new(
        size: impl Into<Vec2>,
        default_alive_cell_positions: Option<Vec<(i32, i32)>>,
    ) -> Self {
        let mut field = Field {
            stdout: stdout(),
            size: size.into(),
            is_auto_play_enabled: false,
            step_count: 0,
            cells: HashSet::new(),
            backuped_cells: None,
        };

        if let Some(positions) = default_alive_cell_positions {
            field.add_cells(positions);
        }

        field
    }

    pub fn add_cell(&mut self, spawn_position: &(impl Into<Vec2> + Copy)) {
        self.cells.insert(Cell::new(*spawn_position));
    }

    pub fn add_cells(&mut self, spawn_positions: Vec<(i32, i32)>) {
        for position in spawn_positions {
            self.add_cell(&position);
        }
    }

    pub fn remove_cell(&mut self, position: &(impl Into<Vec2> + Copy)) {
        let cell = Cell::new(*position);
        self.cells.remove(&cell);
    }

    pub fn clear(&mut self) {
        self.cells = HashSet::new();
        self.backuped_cells = None;
        self.step_count = 0;
    }

    pub fn toggle_auto_play(&mut self) {
        self.is_auto_play_enabled = !self.is_auto_play_enabled;
    }

    pub fn backup(&mut self) {
        if self.backuped_cells.is_none() {
            self.backuped_cells = Some(self.cells.clone());
        }
    }

    pub fn restore_backup(&mut self) {
        if let Some(backuped_cells) = self.backuped_cells.clone() {
            self.cells = backuped_cells;
            self.backuped_cells = None;
            self.step_count = 0;
        }
    }

    pub fn step(&mut self) {
        self.backup();
        self.step_count += 1;

        let mut target_cells = self
            .cells
            .iter()
            .flat_map(|cell| {
                DIRECTIONS
                    .iter()
                    .map(move |&offset| Cell::new(cell.position + offset.into()))
            })
            .collect::<HashSet<Cell>>();

        target_cells.extend(self.cells.iter().cloned());

        self.cells = target_cells
            .into_iter()
            .filter(|cell| cell.is_next_alive(self))
            .collect();
    }

    pub fn draw(&mut self) -> Result<()> {
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                execute!(self.stdout, cursor::MoveTo(x as u16, y as u16))?;
                let color = if self.is_auto_play_enabled {
                    Color::White
                } else {
                    Color::Green
                };
                let symbol = if self.cells.contains(&Cell::new((x, y))) {
                    "■".with(color)
                } else {
                    "⋅".with(Color::DarkGrey)
                };
                execute!(self.stdout, Print(symbol))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::IntoCoordinates;

    #[test]
    fn test_new_field() {
        let field = Field::new((10, 10), None);
        assert_eq!(field.size, Vec2::new(10, 10));
        assert!(field.cells.is_empty());
        assert!(!field.is_auto_play_enabled);
    }

    #[test]
    fn test_add_cell() {
        let mut field = Field::new((10, 10), None);
        field.add_cell(&(5, 5));
        assert!(field.cells.contains(&Cell::new((5, 5))));
    }

    #[test]
    fn test_remove_cell() {
        let mut field = Field::new((10, 10), None);
        field.add_cell(&(5, 5));
        field.remove_cell(&(5, 5));
        assert!(!field.cells.contains(&Cell::new((5, 5))));
    }

    #[test]
    fn test_reset() {
        let mut field = Field::new((10, 10), None);
        field.add_cell(&(5, 5));
        field.clear();
        assert!(field.cells.is_empty());
    }

    #[test]
    fn test_toggle_auto_step() {
        let mut field = Field::new((10, 10), None);
        assert!(!field.is_auto_play_enabled);
        field.toggle_auto_play();
        assert!(field.is_auto_play_enabled);
    }

    #[test]
    fn test_step() {
        let mut field = Field::new(
            (10, 10),
            Some(vec![vec![1, 0, 0, 0], vec![1, 0, 0, 0], vec![0, 1, 1, 1]].into_coordinates()),
        );
        let expected_alive_positions =
            vec![vec![0, 0, 0], vec![1, 0, 1], vec![0, 1, 1], vec![0, 0, 1]].into_coordinates();

        field.step();

        for position in expected_alive_positions {
            assert!(field.cells.contains(&Cell::new(position)));
        }
    }
}
