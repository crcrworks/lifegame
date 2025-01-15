mod cell;
mod field;

use crossterm::{
    cursor,
    event::{
        self, poll, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    style::{self, Color, Stylize},
    terminal,
};
use eyre::Result;
use field::Field;
use std::time::Duration;

pub static REDRAW_DURATION: Duration = Duration::from_millis(100);
pub static DIRECTIONS: [(i32, i32); 8] = [
    (0, -1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, 1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
];

type Vec2<T = i32> = vector2d::Vector2D<T>;

#[allow(dead_code)]
trait IntoCoordinates {
    fn into_coordinates(self) -> Vec<(i32, i32)>;
}

impl IntoCoordinates for Vec<Vec<i32>> {
    fn into_coordinates(self) -> Vec<(i32, i32)> {
        self.iter()
            .enumerate()
            .flat_map(|(i, row)| {
                row.iter().enumerate().filter_map(move |(j, &value)| {
                    if value == 1 {
                        Some((j as i32, i as i32))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

fn main() -> Result<()> {
    terminal::enable_raw_mode()?;
    let size = Vec2::from(terminal::size()?);
    let mut field = Field::new(size - Vec2::new(0, 1), None);

    execute!(
        field.stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All),
        event::EnableMouseCapture
    )?;

    loop {
        field.draw()?;

        let color = if field.is_auto_play_enabled {
            Color::Grey
        } else {
            Color::Green
        };
        execute!(
            field.stdout,
            cursor::MoveTo(0, (field.size.y + 1) as u16),
            style::Print(
                format!(
                "{:4} | L-click: put | R-click: remove | Space: play/pause | s: step | r: restore | c: clear | q: quit", field.step_count)
                    .with(color)
            )
        )?;

        if field.is_auto_play_enabled {
            field.step();
        }

        if poll(REDRAW_DURATION)? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(' '),
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    field.toggle_auto_play();
                    if field.is_auto_play_enabled {
                        execute!(field.stdout, event::DisableMouseCapture)?;
                    } else {
                        execute!(field.stdout, event::EnableMouseCapture)?;
                    }
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Char('s'),
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    if !field.is_auto_play_enabled {
                        field.step();
                    }
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Char('r'),
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    if !field.is_auto_play_enabled {
                        field.restore_backup();
                    }
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    if !field.is_auto_play_enabled {
                        field.clear();
                    }
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    break;
                }

                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(button_type),
                    column,
                    row,
                    ..
                }) if !field.is_auto_play_enabled => match button_type {
                    MouseButton::Left => field.add_cell(&(column, row)),
                    MouseButton::Right => field.remove_cell(&(column, row)),
                    _ => {}
                },

                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Drag(button_type),
                    column,
                    row,
                    ..
                }) if !field.is_auto_play_enabled => match button_type {
                    MouseButton::Left => field.add_cell(&(column, row)),
                    MouseButton::Right => field.remove_cell(&(column, row)),
                    _ => {}
                },

                _ => {}
            }
        }
    }

    execute!(
        field.stdout,
        cursor::Show,
        event::DisableMouseCapture,
        terminal::LeaveAlternateScreen,
    )?;
    Ok(())
}
