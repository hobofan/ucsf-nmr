// use insta::assert_debug_snapshot;
use ucsf_nmr::UcsfFile;

#[test]
fn data_continous_2d_simple() {
    let contents = include_bytes!("./data/15n_hsqc.ucsf");

    let (_, file) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    let _ = file.data_continous();
    // assert_debug_snapshot!(value);
}

#[test]
fn data_continous_2d_padded() {
    let contents = include_bytes!("./data/Nhsqc_highres_600MHz.ucsf");

    let (_, file) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    // basic check that we don't panic
    file.data_continous();
}
