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
//!     for ((i_axis_1, i_axis_2), value) in tile.iter_with_abolute_pos().as_2d() {
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

    /// Returns an iterator over all tiles in the file.
    pub fn tiles(&self) -> Tiles<'_> {
        Tiles::for_file(&self)
    }

    /// Returns the amount of tiles along each axis.
    pub fn axis_tiles(&self) -> Vec<usize> {
        self.axis_headers
            .iter()
            .map(|axis| axis.num_tiles() as usize)
            .collect()
    }

    /// Returns the amount of data points in a tile along all axis.
    pub fn axis_tile_sizes(&self) -> Vec<usize> {
        self.axis_headers
            .iter()
            .map(|axis| axis.tile_size as usize)
            .collect()
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
        let total_size = Self::calculate_data_size(&self.axis_headers);
        let mut data = [0f32].repeat(total_size);

        for tile in self.tiles() {
            for (axis_indices, value) in tile.iter_with_abolute_pos() {
                let pos = multi_dim_position(&self.axis_sizes(), &axis_indices);
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

    /// Check whether the tile with index `tile_n` has padding along this axis.
    pub fn tile_is_padded(&self, tile_n: usize) -> bool {
        let num_full_tiles = self.data_points / self.tile_size;

        tile_n >= num_full_tiles as usize
    }

    /// Returns the amount of padding for tile with index `tile_n` along this axis.
    pub fn tile_padding(&self, tile_n: usize) -> u32 {
        match self.tile_is_padded(tile_n) {
            false => 0,
            true => self.padded_size() - self.data_points,
        }
    }
}

pub struct Tile<'a> {
    /// Amount of data points along each axis in this tile.
    pub axis_lengths: Vec<usize>,
    /// Index of first element of axis 1 (in relation to total axis).
    // pub axis_1_start: usize,
    /// Index of first element of axis 2 (in relation to total axis).
    // pub axis_2_start: usize,
    /// Index of first element of axis 2 (in relation to total axis).
    pub axis_starts: Vec<usize>,
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

impl<'a> AbsolutePosValIter<'a> {
    pub fn as_2d(&'a mut self) -> AbsolutePosValIter2D<'a> {
        AbsolutePosValIter2D { iter: self }
    }

    pub fn as_3d(&'a mut self) -> AbsolutePosValIter3D<'a> {
        AbsolutePosValIter3D { iter: self }
    }

    pub fn as_4d(&'a mut self) -> AbsolutePosValIter4D<'a> {
        AbsolutePosValIter4D { iter: self }
    }
}

impl<'a> Iterator for AbsolutePosValIter<'a> {
    type Item = (Vec<usize>, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index >= self.tile.data().len() {
            return None;
        }

        // Position relative to the current tile
        let axis_rel = multi_dim_index(&self.tile.axis_lengths, self.next_index);
        // Absolute position
        let axis_abs: Vec<_> = axis_rel
            .iter()
            .zip(&self.tile.axis_starts)
            .map(|(axis_relative, axis_start)| axis_relative + axis_start)
            .collect();

        let val = self.tile.data()[self.next_index];
        self.next_index += 1;
        Some(((axis_abs), val))
    }
}

pub struct AbsolutePosValIter2D<'a> {
    iter: &'a mut AbsolutePosValIter<'a>,
}

impl<'a> Iterator for AbsolutePosValIter2D<'a> {
    type Item = ((usize, usize), f32);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(axis_abs, val)| ((axis_abs[0], axis_abs[1]), val))
    }
}

pub struct AbsolutePosValIter3D<'a> {
    iter: &'a mut AbsolutePosValIter<'a>,
}

impl<'a> Iterator for AbsolutePosValIter3D<'a> {
    type Item = ((usize, usize, usize), f32);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(axis_abs, val)| ((axis_abs[0], axis_abs[1], axis_abs[2]), val))
    }
}

pub struct AbsolutePosValIter4D<'a> {
    iter: &'a mut AbsolutePosValIter<'a>,
}

impl<'a> Iterator for AbsolutePosValIter4D<'a> {
    type Item = ((usize, usize, usize, usize), f32);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(axis_abs, val)| ((axis_abs[0], axis_abs[1], axis_abs[2], axis_abs[3]), val))
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
        let tiles_per_axis = self.file.axis_tiles();
        let tiles_total: usize = tiles_per_axis.iter().product();
        if tiles_total <= self.next_index {
            return None;
        }

        let tile_indices = multi_dim_index(&tiles_per_axis, self.next_index);

        // Size of a normal (unpadded) tile
        let axis_tile_sizes = self.file.axis_tile_sizes();
        // Size of this tile (without padding)
        let this_tile_axis_lens: Vec<_> = axis_tile_sizes
            .iter()
            .zip(&tile_indices)
            .zip(&self.file.axis_headers)
            .map(|((tile_size, tile_index), axis_header)| {
                (*tile_size as u32 - axis_header.tile_padding(*tile_index)) as usize
            })
            .collect();

        let axis_starts: Vec<_> = axis_tile_sizes
            .iter()
            .zip(tile_indices)
            .map(|(tile_size, tile_index)| tile_size * tile_index)
            .collect();

        let tile_data_points: usize = this_tile_axis_lens.iter().product();

        let data_range_start = tile_data_points * self.next_index;
        let data_range_end = data_range_start + tile_data_points;

        self.next_index += 1;
        Some(Tile {
            axis_lengths: this_tile_axis_lens,
            axis_starts,
            data: &self.file.data[data_range_start..data_range_end],
        })
    }
}

/// Calculate the position in a flat array from multi-dimension-index and dimension sizes.
fn multi_dim_position(sizes: &[usize], indices: &[usize]) -> usize {
    assert!(sizes.len() == indices.len());

    let mut pos = 0;
    for dim in 0..sizes.len() {
        let mut subdimensions_size = 0;
        let subdimensions = sizes.len() - (dim + 1);
        if subdimensions >= 1 {
            subdimensions_size = sizes[(sizes.len() - subdimensions)..(sizes.len())]
                .iter()
                .product();
        }

        let dim_index = indices[dim];
        match subdimensions_size {
            0 => pos += dim_index,
            subdimensions_size => pos += dim_index * subdimensions_size,
        }
    }

    pos
}

fn multi_dim_index(sizes: &[usize], pos: usize) -> Vec<usize> {
    let mut indices = [0usize].repeat(sizes.len());
    // TODO: implement in generic way
    match sizes.len() {
        2 => {
            indices[0] = pos / sizes[1];
            indices[1] = pos % sizes[1];
        }
        3 => {
            indices[0] = pos / (sizes[1] * sizes[2]);
            indices[1] = (pos % (sizes[1] * sizes[2])) / sizes[2];
            indices[2] = (pos % (sizes[1] * sizes[2])) % sizes[2];
        }
        _ => unimplemented!(),
    }

    indices
}

#[cfg(test)]
mod test {
    #[test]
    fn multi_dim_position() {
        let f = super::multi_dim_position;
        assert_eq!(f(&[3, 3], &[1, 0]), 3);
        assert_eq!(f(&[3, 3], &[1, 1]), 4);
        assert_eq!(f(&[3, 3], &[1, 2]), 5);
        assert_eq!(f(&[3, 3], &[2, 0]), 6);

        assert_eq!(f(&[4, 3, 3], &[2, 0, 0]), 18);
        assert_eq!(f(&[4, 3, 3], &[3, 2, 1]), 34);
    }

    #[test]
    fn multi_dim_index() {
        let f = super::multi_dim_index;

        assert_eq!(f(&[4, 3, 3], 18), vec![2, 0, 0]);
        assert_eq!(f(&[4, 3, 3], 34), vec![3, 2, 1]);

        assert_eq!(f(&[4, 3, 2], 18), vec![3, 0, 0]);
        assert_eq!(f(&[4, 3, 2], 19), vec![3, 0, 1]);
        assert_eq!(f(&[4, 3, 2], 20), vec![3, 1, 0]);
        assert_eq!(f(&[4, 3, 2], 21), vec![3, 1, 1]);
    }
}
