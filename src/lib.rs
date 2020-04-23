///! The implemenation follows the description of the format outlined at
///! <https://www.cgl.ucsf.edu/home/sparky/manual/files.html#UCSFFormat>
use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::number::complete::{be_f32, be_u16, be_u32, be_u8};
use nom::sequence::tuple;
use nom::IResult;
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
    pub data: Vec<u8>,
}

impl UcsfFile {
    fn calculate_data_size(axis_headers: &[AxisHeader]) -> usize {
        // * 4 as each data point is a f32
        axis_headers
            .iter()
            .map(|axis| axis.data_points as usize)
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

        Ok((
            rem,
            Self {
                header,
                axis_headers,
                data: data.to_vec(),
            },
        ))
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
}
