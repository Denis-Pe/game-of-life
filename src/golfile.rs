/*!
For interacting with files representing a Game of Life

Preferred extension being .gol

(probably overengineered though it is very fun LOL)

# File Format

## Prelude

### First 4 Bytes
Characters `gol!`
in ASCII encoding, which would be `0x67 0x6F 0x6C 0x21`

### Next 4 Bytes
The size of the grid in two
u16 numbers, first the x and then the y

### Next 4 Bytes
The updates per second
in one f32 number

### Next 4 Bytes
RGBA representing the color of the grid

### Next 4 Bytes
Square size, an f32

### Next 4 Bytes
RGBA representing the color of the square when it's "off"

### Next 4 Bytes
RGBA representing the color of the square when it's "on"

### Last 7 Bytes
Represent the end of the Prelude,
they should be an empty byte followed by `\gol!/`
in ASCII encoding, or `0x00` followed by `0x5C 0x67 0x6F 0x6C 0x21 0x2F`

## Main Content

Width of grid * height of grid of booleans
representing all the squares.
All the squares in the top row
from left to right, then the second row and etcetera

# Notes

- I decided to make it completely big-endian,
    that way moving files accross different endianness works
    since the program will read big-endian regardless of host endianness,
    but it also helps when seeing the files in an editor to not
    think in backward bytes
*/

use std::array::TryFromSliceError;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

const PRELUDE_LENGTH: usize =
4+4+4+4
+4+4+4
+7;

#[derive(Debug)]
pub enum GOLFileError {
    NotValidFile,
    UnexpectedEndOfBytes,
    IOError(io::Error),
}

impl From<io::Error> for GOLFileError {
    fn from(error: io::Error) -> Self {
        Self::IOError(error)
    }
}

impl From<TryFromSliceError> for GOLFileError {
    fn from(_: TryFromSliceError) -> Self {
        GOLFileError::NotValidFile
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RGBA(pub u8, pub u8, pub u8, pub u8);

#[derive(Debug, Clone)]
pub struct GOLFile {
    pub squares: Vec<bool>,
    pub squares_x: u16,
    pub squares_y: u16,
    pub updates_sec: f32,
    pub grid_color: RGBA,
    pub square_size: f32,
    pub square_color_off: RGBA,
    pub square_color_on: RGBA,
}

impl GOLFile {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, GOLFileError> {
        let mut output = Self {
            squares: Vec::new(),
            squares_x: 0,
            squares_y: 0,
            updates_sec: 0.0,
            grid_color: RGBA(0, 0, 0, 0),
            square_size: 0.0,
            square_color_off: RGBA(0, 0, 0, 0),
            square_color_on: RGBA(0, 0, 0, 0),
        };

        let bytes = fs::read(path)?;

        // START THE PARSING AND MULTIPLE CHECKS

        let first_four: [u8; 4] = bytes[0..4].try_into()?;

        if first_four != *b"gol!" {
            return Err(GOLFileError::NotValidFile);
        }

        output.squares_x = u16::from_be_bytes(bytes[4..6].try_into()?);

        output.squares_y = u16::from_be_bytes(bytes[6..8].try_into()?);

        if output.squares_x == 0 || output.squares_y == 0 {
            return Err(GOLFileError::NotValidFile);
        }

        output.updates_sec = f32::from_be_bytes(bytes[8..12].try_into()?);

        if output.updates_sec == 0.0 {
            return Err(GOLFileError::NotValidFile);
        }

        output.grid_color = RGBA(bytes[12], bytes[13], bytes[14], bytes[15]);

        output.square_size = f32::from_be_bytes(bytes[16..20].try_into()?);

        output.square_color_off = RGBA(bytes[20], bytes[21], bytes[22], bytes[23]);
        output.square_color_on = RGBA(bytes[24], bytes[25], bytes[26], bytes[27]);

        if bytes[28] != 0 {
            return Err(GOLFileError::NotValidFile);
        }

        let last_six: [u8; 6] = bytes[29..35].try_into()?; // last of the prelude

        if last_six != *b"\\gol!/" {
            return Err(GOLFileError::NotValidFile);
        }

        for byte in bytes.iter().skip(PRELUDE_LENGTH) {
            output.squares.push(*byte != 0); // *byte as bool
        }

        if output.squares.len() != output.squares_x as usize * output.squares_y as usize {
            return Err(GOLFileError::UnexpectedEndOfBytes);
        }

        Ok(output)
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        let mut file = File::create(path)?;

        file.write_all(b"gol!")?;

        file.write_all(&self.squares_x.to_be_bytes())?;
        file.write_all(&self.squares_y.to_be_bytes())?;

        file.write_all(&self.updates_sec.to_be_bytes())?;

        file.write_all(&self.grid_color.0.to_be_bytes())?;
        file.write_all(&self.grid_color.1.to_be_bytes())?;
        file.write_all(&self.grid_color.2.to_be_bytes())?;
        file.write_all(&self.grid_color.3.to_be_bytes())?;

        file.write_all(&self.square_size.to_be_bytes())?;

        file.write_all(&self.square_color_off.0.to_be_bytes())?;
        file.write_all(&self.square_color_off.1.to_be_bytes())?;
        file.write_all(&self.square_color_off.2.to_be_bytes())?;
        file.write_all(&self.square_color_off.3.to_be_bytes())?;

        file.write_all(&self.square_color_on.0.to_be_bytes())?;
        file.write_all(&self.square_color_on.1.to_be_bytes())?;
        file.write_all(&self.square_color_on.2.to_be_bytes())?;
        file.write_all(&self.square_color_on.3.to_be_bytes())?;

        file.write_all(b"\0")?;

        file.write_all(b"\\gol!/")?;

        for square in self.squares.iter() {
            file.write_all(&[*square as u8])?;
        }

        Ok(())
    }
}