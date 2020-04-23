use ucsf::UcsfFile;

#[test]
fn correct_num_tiles() {
    let contents = include_bytes!("./data/15n_hsqc.ucsf");

    let (_, contents) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    let tile_count = contents.tiles().count();
    assert_eq!(tile_count, 4);
}
