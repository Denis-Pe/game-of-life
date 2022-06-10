mod golfile;
// use golfile::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RGBA(pub u8, pub u8, pub u8, pub u8);

#[derive(Debug, Clone)]
pub struct Settings {
    squares: Vec<bool>,
    squares_x: u16,
    squares_y: u16,
    updates_sec: f32,
    grid_color: RGBA,
    square_size: f32,
    square_color_off: RGBA,
    square_color_on: RGBA,
}