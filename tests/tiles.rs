use ucsf_nmr::{Tiles, UcsfFile};

#[test]
fn correct_num_tiles_1() {
    let contents = include_bytes!("./data/15n_hsqc.ucsf");

    let (_, contents) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    let tile_count = contents.tiles().count();
    assert_eq!(tile_count, 4);
}

#[test]
fn correct_num_tiles_2() {
    let contents = include_bytes!("./data/Nhsqc_highres_600MHz.ucsf");

    let (_, contents) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    let tile_count = contents.tiles().count();
    assert_eq!(tile_count, 20);
}

#[test]
fn correct_tiles_absolute_positions() {
    let contents = include_bytes!("./data/Nhsqc_highres_600MHz.ucsf");

    let (_, contents) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    let mut tiles = contents.tiles();

    let assert_absolute_pos = |tiles: &mut Tiles, absolute_pos| {
        assert_eq!(
            absolute_pos,
            tiles
                .next()
                .unwrap()
                .iter_with_abolute_pos()
                .next()
                .unwrap()
                .0
        );
    };

    assert_absolute_pos(&mut tiles, (0, 0));
    assert_absolute_pos(&mut tiles, (0, 64));
    assert_absolute_pos(&mut tiles, (0, 128));
    assert_absolute_pos(&mut tiles, (0, 192));
    assert_absolute_pos(&mut tiles, (0, 256));
    assert_absolute_pos(&mut tiles, (128, 0));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 128));
    assert_absolute_pos(&mut tiles, (128, 192));
    assert_absolute_pos(&mut tiles, (128, 256));
    assert_absolute_pos(&mut tiles, (256, 0));
    assert_absolute_pos(&mut tiles, (256, 64));
    assert_absolute_pos(&mut tiles, (256, 128));
    assert_absolute_pos(&mut tiles, (256, 192));
    assert_absolute_pos(&mut tiles, (256, 256));
    assert_absolute_pos(&mut tiles, (384, 0));
    assert_absolute_pos(&mut tiles, (384, 64));
    assert_absolute_pos(&mut tiles, (384, 128));
    assert_absolute_pos(&mut tiles, (384, 192));
    assert_absolute_pos(&mut tiles, (384, 256));
}
