//! The implemenation follows the description of the format outlined at
//! <https://www.cgl.ucsf.edu/home/sparky/manual/files.html#UCSFFormat>
//!
//! ## Usage
//!
//! Reading a spectrum from a file:
//! ```
//! use std::fs;
//! use ucsf_nmr::UcsfFile;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
//!    let file_bytes = fs::read("./tests/data/15n_hsqc.ucsf")?;
//!    // _remaining_bytes should be empty and can usually be discarded.
//!    // ucsf_file contains our data of interest
//!    let (_remaining_bytes, ucsf_file) = UcsfFile::parse(&file_bytes)?;
//!    Ok(())
//! }
//! ```
//!
//! Iterate over all data points in the file via tiles:
//! ```
//! # use std::fs;
//! # use ucsf_nmr::UcsfFile;
//! #
//! # fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
//! #   let file_bytes = fs::read("./tests/data/15n_hsqc.ucsf")?;
//! #   let (_remaining_bytes, ucsf_file) = UcsfFile::parse(&file_bytes)?;
//! #
//!   for tile in ucsf_file.tiles() {
//!     for ((i_axis_1, i_axis_2), value) in tile.iter_with_abolute_pos() {
//!       // i_axis_1 contains coordinate of data point on first axis
//!       // i_axis_2 contains coordinate of data point on first axis
//!       // value contains coordinate of data point on first axis
//!       format!("({},{}) : {}", i_axis_1, i_axis_2, value);
//!     }
//!   }
//! #
//! #   Ok(())
//! # }
//! ```
use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::number::complete::{be_f32, be_u16, be_u32, be_u8};
use nom::sequence::tuple;
use nom::IResult;
use std::convert::TryInto;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum UcsfError {
    #[error("Unsupported format version. Currently the parser only supports format version 2.")]
    UnsupportedFormat,
    #[error("Unsupported number of components. Currently the parser only supports files with a number of1 components per data point (= Real).")]
    UnsupportedComponents,
    #[error("Failed to parse")]
    Parsing,
}

#[derive(Debug, Clone)]
pub struct UcsfFile {
    pub header: Header,
    pub axis_headers: Vec<AxisHeader>,
    pub data: Vec<f32>,
}

impl UcsfFile {
    fn calculate_data_size(axis_headers: &[AxisHeader]) -> usize {
        // * 4 as each data point is a f32
        axis_headers
            .iter()
            .map(|axis| axis.padded_size() as usize)
            .product::<usize>()
            * 4
    }

    fn parse_data_raw(input: &[u8], size: usize) -> IResult<&[u8], &[u8]> {
        take(size)(input)
    }

    pub fn parse(input: &[u8]) -> Result<(&[u8], Self), UcsfError> {
        let (mut rem, header) = Header::parse(&input)?;
        let mut axis_headers = vec![];
        for _ in 0..header.dimensions {
            let (_rem, axis_header) = AxisHeader::parse(&rem)?;
            rem = _rem;
            axis_headers.push(axis_header);
        }

        let data_size = Self::calculate_data_size(&axis_headers);
        let (rem, data) = Self::parse_data_raw(rem, data_size).map_err(|_| UcsfError::Parsing)?;
        let float_data: Vec<f32> = data
            .chunks(4)
            .map(|chunk| f32::from_be_bytes(chunk.try_into().unwrap()))
            .collect();

        Ok((
            rem,
            Self {
                header,
                axis_headers,
                data: float_data,
            },
        ))
    }

    /// Returns the amount of data points along `axis`.
    pub fn axis_data_points(&self, axis: usize) -> u32 {
        self.axis_headers[axis].data_points
    }

    /// Returns the amount of tiles along axis `axis`.
    pub fn axis_tiles(&self, axis: usize) -> u32 {
        self.axis_headers[axis].num_tiles()
    }

    /// Returns the amount of data points in a tile along `axis`.
    pub fn axis_tile_size(&self, axis: usize) -> u32 {
        self.axis_headers[axis].tile_size
    }

    /// Returns an iterator over all tiles in the file.
    pub fn tiles(&self) -> Tiles<'_> {
        Tiles::for_file(&self)
    }

    /// Returns the sizes for all axis.
    ///
    /// Can be used together with `.data_continous()` to use the data
    /// with multidimensional array types from other crates.
    pub fn axis_sizes(&self) -> Vec<usize> {
        self.axis_headers
            .iter()
            .map(|axis| axis.data_points as usize)
            .collect()
    }

    /// Construct a Vec where the data is layed out continously per-axis.
    ///
    /// This provides an alternative way to accessing the data in its native
    /// tile-layout.
    pub fn data_continous(&self) -> Vec<f32> {
        let total_size = self.data.len();
        let mut data = [0f32].repeat(total_size);

        for tile in self.tiles() {
            for ((i_axis_1, i_axis_2), value) in tile.iter_with_abolute_pos() {
                let pos = i_axis_1 * (self.axis_data_points(1) as usize) + i_axis_2;
                data[pos] = value;
            }
        }
        data
    }

    /// Returns the lower and upper bounds of the data.
    pub fn bounds(&self) -> (f32, f32) {
        let mut sorted_data = self.data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let min_val: f32 = *sorted_data.first().unwrap();
        let max_val: f32 = *sorted_data.last().unwrap();

        (min_val, max_val)
    }
}

/// 180 byte header
///
/// ### Format
///
/// - 10 bytes fixed header (`UCSG NMR  `)
/// - [Dimensions](#structfield.dimensions)
/// - [Number of components](#structfield.components)
/// - [Format version](#structfield.format_version)
/// - [Remaining unspecified bytes](#structfield.remainder)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Number of dimensions in the spectrum.
    ///
    /// ### Format
    /// Bytes: 10-10
    pub dimensions: u8,
    /// Number of components for each data point.
    ///
    /// 1 = Real
    /// 2 = Imaginary
    ///
    /// ### Format
    /// Bytes: 11-11
    pub components: u8,
    /// Format version.
    ///
    /// 2 is currently the only supported value.
    ///
    /// ### Format
    /// Bytes: 12-13
    pub format_version: u16,
    /// Remaining unspecified bytes.
    ///
    /// Often filled with date of recording, etc.
    ///
    /// ### Format
    /// Bytes: 14-179
    pub remainder: Vec<u8>,
}

impl Header {
    fn parse_raw(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8], u8, u8, u16, &[u8])> {
        tuple((
            tag(b"UCSF NMR"),
            take(2u8),
            be_u8,
            be_u8,
            be_u16,
            take(166u8),
        ))(input)
    }

    pub fn parse(input: &[u8]) -> Result<(&[u8], Self), UcsfError> {
        let (rem, res) = Self::parse_raw(input).map_err(|_| UcsfError::Parsing)?;

        let map = |(
            _magic_string,
            _magic_strimg_rem,
            dimensions,
            components,
            format_version,
            remainder,
        ): (_, _, _, _, _, &[u8])| {
            if components != 1 {
                return Err(UcsfError::UnsupportedComponents);
            }
            if format_version != 2 {
                return Err(UcsfError::UnsupportedFormat);
            }

            Ok((
                rem,
                Self {
                    dimensions,
                    components,
                    format_version,
                    remainder: remainder.to_vec(),
                },
            ))
        };

        map(res)
    }
}

/// 128 byte axis header
#[derive(Debug, Clone, PartialEq)]
pub struct AxisHeader {
    /// Nucleus name (1H, 13C, 15N, 31P, ...)
    ///
    /// ### Format
    /// Bytes 0-7
    pub nucleus_name: String,
    /// Number of data points along this axis.
    ///
    /// ### Format
    /// Bytes 8-11 (followed by 4 unknown bytes)
    pub data_points: u32,
    /// Tile size along this axis.
    ///
    /// ### Format
    /// Bytes 16-19
    pub tile_size: u32,
    /// Spectrometer frequency for this nucleus (MHz)
    ///
    /// ### Format
    /// Bytes 20-23
    pub frequency: f32,
    /// Spectral width (Hz)
    ///
    /// ### Format
    /// Bytes 24-27
    pub spectral_width: f32,
    /// Center of data (ppm)
    ///
    /// ### Format
    /// Bytes 28-31
    pub center: f32,
    /// Remaining unspecified bytes
    ///
    /// ### Format
    /// Bytes: 32-127
    pub remainder: Vec<u8>,
}

impl AxisHeader {
    fn parse_raw(input: &[u8]) -> IResult<&[u8], (&[u8], u32, &[u8], u32, f32, f32, f32, &[u8])> {
        tuple((
            take(8u8),
            be_u32,
            take(4u8),
            be_u32,
            be_f32,
            be_f32,
            be_f32,
            take(96u8),
        ))(input)
    }

    pub fn parse(input: &[u8]) -> Result<(&[u8], Self), UcsfError> {
        let (rem, res) = Self::parse_raw(input).map_err(|_| UcsfError::Parsing)?;

        let map = |(
            nucleus_name,
            data_points,
            _unknown,
            tile_size,
            frequency,
            spectral_width,
            center,
            remainder,
        ): (&[u8], _, _, _, _, _, _, &[u8])| {
            let nucleus_name =
                String::from_utf8_lossy(nucleus_name.split(|n| *n == 0u8).next().unwrap())
                    .trim_end()
                    .to_owned();
            Ok((
                rem,
                Self {
                    nucleus_name,
                    data_points,
                    tile_size,
                    frequency,
                    spectral_width,
                    center,
                    remainder: remainder.to_vec(),
                },
            ))
        };

        map(res)
    }

    /// Returns the amount of tiles along this axis.
    pub fn num_tiles(&self) -> u32 {
        // We are adding `self.tile_size - 1`, to ensure we always round up
        // for zero-padded tiles
        (self.data_points + self.tile_size - 1) / self.tile_size
    }

    /// Returns the size of the axis including zero-padding.
    ///
    /// Useful for determining the expected size of the file.
    pub fn padded_size(&self) -> u32 {
        self.num_tiles() * self.tile_size
    }
}

pub struct Tile<'a> {
    /// Amount of data points along axis 1 in this tile.
    pub axis_1_len: usize,
    /// Amount of data points along axis 2 in this tile.
    pub axis_2_len: usize,
    /// Index of first element of axis 1 (in relation to total axis).
    pub axis_1_start: usize,
    /// Index of first element of axis 2 (in relation to total axis).
    pub axis_2_start: usize,
    /// View into underlying data
    pub data: &'a [f32],
}

impl<'a> Tile<'a> {
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Iterate over the values in a tile with their absolute position in the spectrum.
    ///
    /// **No specific order of the values should be assumes, which is why the position is provided
    /// in the iterator**
    pub fn iter_with_abolute_pos(&self) -> AbsolutePosValIter<'_> {
        AbsolutePosValIter {
            tile: self,
            next_index: 0,
        }
    }
}

pub struct AbsolutePosValIter<'a> {
    tile: &'a Tile<'a>,
    next_index: usize,
}

impl<'a> Iterator for AbsolutePosValIter<'a> {
    type Item = ((usize, usize), f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index >= self.tile.data().len() {
            return None;
        }

        let axis_1_rel = self.next_index / self.tile.axis_2_len;
        let axis_2_rel = self.next_index % self.tile.axis_2_len;

        let axis_1_abs = axis_1_rel + self.tile.axis_1_start;
        let axis_2_abs = axis_2_rel + self.tile.axis_2_start;

        let val = self.tile.data()[self.next_index];
        self.next_index += 1;
        Some(((axis_1_abs, axis_2_abs), val))
    }
}

pub struct Tiles<'a> {
    next_index: usize,
    file: &'a UcsfFile,
}

impl<'a> Tiles<'a> {
    pub fn for_file(file: &'a UcsfFile) -> Self {
        Self {
            next_index: 0,
            file,
        }
    }
}

impl<'a> Iterator for Tiles<'a> {
    type Item = Tile<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let tiles_axis_1 = self.file.axis_tiles(0) as usize;
        let tiles_axis_2 = self.file.axis_tiles(1) as usize;
        let tiles_total = tiles_axis_1 * tiles_axis_2;
        if tiles_total <= self.next_index {
            return None;
        }

        let tile_index_1 = self.next_index / tiles_axis_2;
        let tile_index_2 = self.next_index % tiles_axis_2;

        let axis_1_len = self.file.axis_tile_size(0) as usize;
        let axis_2_len = self.file.axis_tile_size(1) as usize;

        let axis_1_start = axis_1_len * tile_index_1;
        let axis_2_start = axis_2_len * tile_index_2;

        let tile_data_points = axis_1_len * axis_2_len;

        let data_range_start = tile_data_points * self.next_index;
        let data_range_end = data_range_start + tile_data_points;

        self.next_index += 1;
        Some(Tile {
            axis_1_len,
            axis_2_len,
            axis_1_start,
            axis_2_start,
            data: &self.file.data[data_range_start..data_range_end],
        })
    }
}
