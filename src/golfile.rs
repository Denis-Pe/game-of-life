//! For interacting with files representing a Game of Life
//!
//! File extension being .gol
//!
//! (probably overengineered though it is very fun LOL)
//!
//! # File Format
//! ## Prelude
//!
//! The first 4 bytes of the file must represent the characters `gol!`
//! in ASCII encoding, which would be `0x67 0x6F 0x6C 0x21`
//!
//! The next 4 bytes should represent the size of the grid in two
//! u16 numbers, first the x/columns and then the y/rows
//!
//! The next 4 bytes should represent the updates per second
//! in one f32 number
//!
//! The last 7 bytes should represent the end of the Prelude,
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
//! exactly divisible by 8, but that is why
//! I keep track of the size of the grid.

use std::array::TryFromSliceError;
use std::fs::File;
use std::io::{self, Write};
use std::io::Read;
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

type Result<T> = std::result::Result<T, FileReadError>;

pub struct GOLFile {
    pub squares: Vec<u8>,
    pub squares_x: u16,
    pub squares_y: u16,
    pub updates_sec: f32,
}

impl GOLFile {
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let mut output = Self {
            squares: Vec::new(),
            squares_x: 0,
            squares_y: 0,
            updates_sec: 0.0,
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
        
        output.squares_x |= (bytes[4] as u16) << 8;
        output.squares_x |= bytes[5] as u16;

        output.squares_y |= (bytes[6] as u16) << 8;
        output.squares_y |= bytes[7] as u16;

        if output.squares_x == 0 || output.squares_y == 0 {
            return Err(FileReadError::NotValidFile)
        }

        output.updates_sec = f32::from_ne_bytes(bytes[8..12].try_into()?);

        if output.updates_sec == 0.0 {
            return Err(FileReadError::NotValidFile)
        }

        if bytes[12] != 0 {
            return Err(FileReadError::NotValidFile)
        }

        let last_six: [u8; 6] = bytes[13..19].try_into()?; // last of the prelude

        if last_six != *b"\\gol!/" {
            return Err(FileReadError::NotValidFile)
        }

        for byte in bytes.iter().skip(19) {
            output.squares.push(*byte);
        }

        if bytes.len() * 8 < (output.squares_x * output.squares_y) as usize {
            return Err(FileReadError::UnexpectedEndOfBytes)
        }

        Ok(output)
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut file = File::create(path)?;

        file.write_all(b"gol!")?;

        file.write_all(&self.squares_x.to_ne_bytes())?;

        file.write_all(&self.squares_y.to_ne_bytes())?;

        file.write_all(&self.updates_sec.to_ne_bytes())?;

        file.write_all(b"\\gol!/")?;

        file.write_all(self.squares.as_slice())?;
        
        Ok(())
    }
}
