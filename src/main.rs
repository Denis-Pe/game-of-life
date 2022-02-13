/*
plans:

struct Grid {
  center_sq
  sq_color
  sq_size: 
  switch: Vec<Square>
  box_border: bool
  updates: f32
}

struct Square {
  on: bool,
  neighbours: u8
}

impl Square {

  // switch is a vector from the struct Grid
  fn update {
    gather data about neighbours

    if self.on {
      if self.neighbours < 2 || self.neighbours > 3 {
        switch.push(self)
      }
    } else if self.neighbours == 3 {
      switch.push(self)
    }
  }
}
*/

use eframe::{epi, egui::*};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 450.0;

#[derive(Default)]
struct Grid {

}

impl epi::App for Grid {
  fn name(&self) -> &str {
    "Conway's Game of Life"
  }

  fn update(&mut self, ctx: &CtxRef, _frame: &mut epi::Frame<'_>) {

  }
}

fn main() {
  let grid = Grid::default();
  let native_options = eframe::NativeOptions {
    always_on_top: false,
    maximized: false,
    decorated: true,
    drag_and_drop_support: false,
    icon_data: None,
    initial_window_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
    resizable: true,
    transparent: false,
  };
  eframe::run_native(Box::new(grid), native_options);
}