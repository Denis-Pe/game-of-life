/*!
For interacting with files representing a Game of Life
and reading those settings in the file into the Settings struct

Preferred extension being .gol

(probably overengineered though it is very fun to dev this LOL)

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

### Next 5 Bytes
How the game should initially look like, which is represented
in an enum kind of matter:
- If the setting is fit the grid to screen, then the entire 5 bytes will be
zeroed
- If the setting is to look at the center with a certain amount of
zoom, the five bytes will look like a zero byte followed
by an f32 of zoom

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

use super::*;

#[rustfmt::skip]
const PRELUDE_LENGTH: usize =
4+4+4+4+
5+4+4+7;

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

impl Settings {
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self, GOLFileError> {
        let mut output = Self {
            squares: Vec::new(),
            squares_x: 0,
            squares_y: 0,
            updates_sec: 0.0,
            background_color: RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            starting_view: StartingView::FitGridToScreen,
            square_color_off: RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            square_color_on: RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
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

        output.background_color = RGBA {
            r: bytes[12],
            g: bytes[13],
            b: bytes[14],
            a: bytes[15],
        };

        output.starting_view = StartingView::from_be_bytes(bytes[16..21].try_into()?)?;

        output.square_color_off = RGBA {
            r: bytes[21],
            g: bytes[22],
            b: bytes[23],
            a: bytes[24],
        };
        output.square_color_on = RGBA {
            r: bytes[25],
            g: bytes[26],
            b: bytes[27],
            a: bytes[28],
        };

        if bytes[29] != 0 {
            return Err(GOLFileError::NotValidFile);
        }

        let last_six: [u8; 6] = bytes[30..36].try_into()?; // last of the prelude

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

    pub fn write_in_file(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        let mut file = File::create(path)?;

        file.write_all(b"gol!")?;

        file.write_all(&self.squares_x.to_be_bytes())?;
        file.write_all(&self.squares_y.to_be_bytes())?;

        file.write_all(&self.updates_sec.to_be_bytes())?;

        file.write_all(&self.background_color.to_be_bytes())?;

        file.write_all(&self.starting_view.to_be_bytes())?;

        file.write_all(&self.square_color_off.to_be_bytes())?;

        file.write_all(&self.square_color_on.to_be_bytes())?;

        file.write_all(b"\0")?;

        file.write_all(b"\\gol!/")?;

        for square in self.squares.iter() {
            file.write_all(&[*square as u8])?;
        }

        Ok(())
    }
}