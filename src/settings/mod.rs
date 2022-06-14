mod golfile;
pub use golfile::*;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGBA {
    pub fn to_f32(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    pub fn from_f32(color: [f32; 4]) -> Self {
        Self {
            r: (color[0] * 255.0) as u8,
            g: (color[1] * 255.0) as u8,
            b: (color[2] * 255.0) as u8,
            a: (color[3] * 255.0) as u8,
        }
    }

    pub fn to_be_bytes(&self) -> [u8; 4] {
        [
            self.r.to_be_bytes()[0],
            self.g.to_be_bytes()[0],
            self.b.to_be_bytes()[0],
            self.a.to_be_bytes()[0],
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StartingView {
    FitGridToScreen,
    Center(f32),
}

impl StartingView {
    pub fn to_be_bytes(self) -> [u8; 5] {
        use StartingView::*;

        match self {
            FitGridToScreen => [0x00, 0x00, 0x00, 0x00, 0x00],
            Center(zoom) => {
                let zoom_bytes = zoom.to_be_bytes();

                [
                    0x01,
                    zoom_bytes[0],
                    zoom_bytes[1],
                    zoom_bytes[2],
                    zoom_bytes[3],
                ]
            }
        }
    }

    pub fn from_be_bytes(bytes: [u8; 5]) -> Result<Self, GOLFileError> {
        match bytes[0] {
            0x00 => Ok(Self::FitGridToScreen),
            0x01 => {
                let zoom = &bytes[1..4];

                Ok(Self::Center(f32::from_be_bytes(match zoom.try_into() {
                    Ok(four_bytes) => four_bytes,
                    Err(_) => return Err(GOLFileError::NotValidFile),
                })))
            }
            _ => Err(GOLFileError::NotValidFile),
        }
    }
}

impl Default for StartingView {
    fn default() -> Self {
        Self::FitGridToScreen
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    /// ### Order
    /// Row major order
    /// i.e. squares\[row\]\[column\]
    squares: Vec<Vec<bool>>, // TODO: ACTUAL USAGE
    squares_x: u16,
    squares_y: u16,
    updates_sec: f32,
    background_color: RGBA,
    starting_view: StartingView,
    square_color_off: RGBA,
    square_color_on: RGBA,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            squares: vec![vec![false; 5]; 5],
            squares_x: 5,
            squares_y: 5,
            updates_sec: 2.0,
            background_color: RGBA {
                r: 4,
                g: 4,
                b: 4,
                a: 255,
            },
            starting_view: StartingView::FitGridToScreen,
            square_color_off: RGBA {
                r: 105,
                g: 105,
                b: 105,
                a: 255,
            },
            square_color_on: RGBA {
                r: 211,
                g: 211,
                b: 211,
                a: 255,
            },
        }
    }
}

impl Settings {
    pub fn squares_x(&self) -> u16 {
        self.squares_x
    }

    pub fn squares_y(&self) -> u16 {
        self.squares_y
    }

    pub fn resize_grid(&mut self, columns: u16, rows: u16) {
        self.squares_x = columns;
        self.squares_y = rows;

        self.squares.clear();

        for row in 0..self.squares_y {
            self.squares.push(Vec::new());
            for _column in 0..self.squares_x {
                self.squares[row as usize].push(false);
            }
        }
    }

    pub fn set_background_color(&mut self, new: RGBA) {
        self.background_color = new;
    }

    pub fn set_sqcolor_off(&mut self, new: RGBA) {
        self.square_color_off = new;
    }

    pub fn set_sqcolor_on(&mut self, new: RGBA) {
        self.square_color_on = new;
    }

    pub fn background_color(&self) -> RGBA {
        self.background_color
    }

    pub fn sqcolor_off(&self) -> RGBA {
        self.square_color_off
    }

    pub fn sqcolor_on(&self) -> RGBA {
        self.square_color_on
    }
}
