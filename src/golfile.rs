//! For interacting with files representing a Game of Life
//!
//! File extension being .gol
//!
//! (probably overengineered though it is very fun LOL)
//!
//! # File Format
//! ## Prelude
//!
//! ### First Four Bytes
//! Characters `gol!`
//! in ASCII encoding, which would be `0x67 0x6F 0x6C 0x21`
//!
//! ### Next 4 Bytes
//! The size of the grid in two
//! u16 numbers, first the x/columns and then the y/rows
//!
//! ### Next 4 Bytes
//! The updates per second
//! in one f32 number
//!
//! ### Next 4 Bytes
//! RGBA representing the color of the grid
//!
//! ### Next 4 Bytes
//! Square size, an f32
//!
//! ### Next 8 Bytes
//! Two RGBAs representing colors of when the
//! square is off and on. Off color first,
//! then on.
//!
//! ### Last 7 Bytes
//! Represent the end of the Prelude,
//! they should be an empty byte followed by `\gol!/` again
//! in ASCII encoding, or `0x5C 0x67 0x6F 0x6C 0x21 0x2F`
//!
//! ## Main Content
//!
//! Literally a bunch of zeroes and ones representing the
//! squares in the grid starting from the top left and
//! going the right and then to the next row,
//! one byte representing 8 of these squares.
//! I am aware the total number of squares may not be
//! perfectly divisible by 8, but that is why
//! I keep track of the size of the grid.

use std::array::TryFromSliceError;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug)]
pub enum FileReadError {
    NotValidFile,
    UnexpectedEndOfBytes,
    IOError(io::Error),
}

impl From<io::Error> for FileReadError {
    fn from(error: io::Error) -> Self {
        Self::IOError(error)
    }
}

impl From<TryFromSliceError> for FileReadError {
    fn from(_: TryFromSliceError) -> Self {
        FileReadError::NotValidFile
    }
}

pub struct RGBA(pub u8, pub u8, pub u8, pub u8);

pub struct GOLFile {
    pub squares: Vec<u8>,
    pub squares_x: u16,
    pub squares_y: u16,
    pub updates_sec: f32,
    pub grid_color: RGBA,
    pub square_size: f32,
    pub square_colors: (RGBA, RGBA),
}

impl GOLFile {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, FileReadError> {
        let mut output = Self {
            squares: Vec::new(),
            squares_x: 0,
            squares_y: 0,
            updates_sec: 0.0,
            grid_color: RGBA(0, 0, 0, 0),
            square_size: 0.0,
            square_colors: (RGBA(0, 0, 0, 0), RGBA(0, 0, 0, 0)),
        };

        let mut file = File::create(path)?;

        let mut bytes = Vec::new();

        match file.read_to_end(&mut bytes) {
            Ok(bytes_read) => bytes_read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                // the documentation says it can usually be retried
                file.read_to_end(&mut bytes)?
            }
            Err(another_error) => return Err(another_error.into()),
        };

        // START THE PARSING AND MULTIPLE CHECKS

        let first_four: [u8; 4] = bytes[0..4].try_into()?;

        if first_four != *b"gol!" {
            return Err(FileReadError::NotValidFile);
        }

        output.squares_x = u16::from_ne_bytes(bytes[4..6].try_into()?);

        output.squares_y = u16::from_ne_bytes(bytes[6..8].try_into()?);

        if output.squares_x == 0 || output.squares_y == 0 {
            return Err(FileReadError::NotValidFile);
        }

        output.updates_sec = f32::from_ne_bytes(bytes[8..12].try_into()?);

        if output.updates_sec == 0.0 {
            return Err(FileReadError::NotValidFile);
        }

        output.grid_color = RGBA(bytes[12], bytes[13], bytes[14], bytes[15]);

        output.square_size = f32::from_ne_bytes(bytes[16..20].try_into()?);

        output.square_colors = (
            RGBA(bytes[20], bytes[21], bytes[22], bytes[23]),
            RGBA(bytes[24], bytes[25], bytes[26], bytes[27]),
        );

        if bytes[28] != 0 {
            return Err(FileReadError::NotValidFile);
        }

        let last_six: [u8; 6] = bytes[29..35].try_into()?; // last of the prelude

        if last_six != *b"\\gol!/" {
            return Err(FileReadError::NotValidFile);
        }

        for byte in bytes.iter().skip(35) {
            output.squares.push(*byte);
        }

        if bytes.len() * 8 < (output.squares_x * output.squares_y) as usize {
            return Err(FileReadError::UnexpectedEndOfBytes);
        }

        Ok(output)
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        let mut file = File::create(path)?;

        file.write_all(b"gol!")?;

        file.write_all(&self.squares_x.to_ne_bytes())?;
        file.write_all(&self.squares_y.to_ne_bytes())?;

        file.write_all(&self.updates_sec.to_ne_bytes())?;

        file.write_all(&self.grid_color.0.to_ne_bytes())?;
        file.write_all(&self.grid_color.1.to_ne_bytes())?;
        file.write_all(&self.grid_color.2.to_ne_bytes())?;
        file.write_all(&self.grid_color.3.to_ne_bytes())?;

        file.write_all(&self.square_size.to_ne_bytes())?;

        file.write_all(&self.square_colors.0.0.to_ne_bytes())?;
        file.write_all(&self.square_colors.0.1.to_ne_bytes())?;
        file.write_all(&self.square_colors.0.2.to_ne_bytes())?;
        file.write_all(&self.square_colors.0.3.to_ne_bytes())?;

        file.write_all(&self.square_colors.1.0.to_ne_bytes())?;
        file.write_all(&self.square_colors.1.1.to_ne_bytes())?;
        file.write_all(&self.square_colors.1.2.to_ne_bytes())?;
        file.write_all(&self.square_colors.1.3.to_ne_bytes())?;

        file.write_all(b"\0")?;

        file.write_all(b"\\gol!/")?;

        file.write_all(self.squares.as_slice())?;

        Ok(())
    }
}
