use float_eq::assert_float_eq;

use ucsf_nmr::{AxisHeader, Header, UcsfError, UcsfFile};

#[test]
fn parse_file() {
    let contents = include_bytes!("./data/15n_hsqc.ucsf");

    let (rem, _) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    assert_eq!(rem.len(), 0);
}

#[test]
fn parse_file_padded() {
    let contents = include_bytes!("./data/Nhsqc_highres_600MHz.ucsf");

    let (rem, _) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    assert_eq!(rem.len(), 0);
}

#[test]
fn parse_header() {
    let contents = include_bytes!("./data/15n_hsqc.ucsf");

    let (_, header) = Header::parse(&contents[..]).expect("Failed parsing");
    assert_eq!(
        Header {
            dimensions: 2,
            components: 1,
            format_version: 2,
            remainder: contents[14..180].to_vec()
        },
        header
    );
}

#[test]
fn parse_header_2() {
    let contents = include_bytes!("./data/Nhsqc_highres_600MHz.ucsf");

    let (_, header) = Header::parse(&contents[..]).expect("Failed parsing");
    assert_eq!(
        Header {
            dimensions: 2,
            components: 1,
            format_version: 2,
            remainder: contents[14..180].to_vec()
        },
        header
    );
}

#[test]
fn parse_header_format_error() {
    let contents = include_bytes!("./data/15n_hsqc.ucsf.invalid_format");

    assert_eq!(
        Err(UcsfError::UnsupportedFormat),
        Header::parse(&contents[..])
    );
}

#[test]
fn parse_axis_header_1() {
    let contents = include_bytes!("./data/15n_hsqc.ucsf");

    let header = AxisHeader::parse(&contents[180..])
        .expect("Failed parsing")
        .1;
    assert_eq!(header.nucleus_name, "15N".to_owned());
    assert_eq!(header.data_points, 256);
    assert_eq!(header.tile_size, 128);
    assert_float_eq!(header.frequency, 60.833f32, ulps <= 1);
    assert_float_eq!(header.spectral_width, 1824.818f32, ulps <= 1);
    assert_float_eq!(header.center, 117.04299f32, ulps <= 1);
}

#[test]
fn parse_axis_header_2() {
    let contents = include_bytes!("./data/15n_hsqc.ucsf");

    let header = AxisHeader::parse(&contents[308..])
        .expect("Failed parsing")
        .1;
    assert_eq!(header.nucleus_name, "1H".to_owned());
    assert_eq!(header.data_points, 352);
    assert_eq!(header.tile_size, 176);
    assert_float_eq!(header.frequency, 600.283f32, ulps <= 1);
    assert_float_eq!(header.spectral_width, 3305.2886f32, ulps <= 1);
    assert_float_eq!(header.center, 8.244598f32, ulps <= 1);
}

#[test]
fn correct_dimensions() {
    let contents = include_bytes!("./data/Nhsqc_highres_600MHz.ucsf");

    let (_, file) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    assert_eq!(file.axis_data_points(0), 512);
    assert_eq!(file.axis_data_points(1), 257);
}

#[test]
fn correct_tile_sizes() {
    let contents = include_bytes!("./data/Nhsqc_highres_600MHz.ucsf");

    let (_, file) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    assert_eq!(file.axis_tile_size(0), 128);
    assert_eq!(file.axis_tile_size(1), 64);
}
