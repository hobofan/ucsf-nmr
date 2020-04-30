use std::ops::Range;

use ucsf_nmr::{Tiles, UcsfFile};

#[test]
fn correct_axis_tiles_1() {
    let contents = include_bytes!("./data/15n_hsqc.ucsf");

    let (_, file) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    assert_eq!(file.axis_tiles()[0], 2);
    assert_eq!(file.axis_tiles()[1], 2);
}

#[test]
fn correct_axis_tiles_padded() {
    let contents = include_bytes!("./data/Nhsqc_highres_600MHz.ucsf");

    let (_, file) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    assert_eq!(file.axis_tiles()[0], 4);
    assert_eq!(file.axis_tiles()[1], 5);
}

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
fn correct_padded_tiles() {
    let contents = include_bytes!("./data/Nhsqc_highres_600MHz.ucsf");

    let (_, contents) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    assert_eq!(false, contents.axis_headers[0].tile_is_padded(0));
    assert_eq!(false, contents.axis_headers[0].tile_is_padded(1));
    assert_eq!(false, contents.axis_headers[0].tile_is_padded(2));
    assert_eq!(false, contents.axis_headers[0].tile_is_padded(3));
    assert_eq!(false, contents.axis_headers[1].tile_is_padded(0));
    assert_eq!(false, contents.axis_headers[1].tile_is_padded(1));
    assert_eq!(false, contents.axis_headers[1].tile_is_padded(2));
    assert_eq!(false, contents.axis_headers[1].tile_is_padded(3));
    assert_eq!(true, contents.axis_headers[1].tile_is_padded(4));

    assert_eq!(0, contents.axis_headers[0].tile_padding(0));
    assert_eq!(0, contents.axis_headers[0].tile_padding(1));
    assert_eq!(0, contents.axis_headers[0].tile_padding(2));
    assert_eq!(0, contents.axis_headers[0].tile_padding(3));
    assert_eq!(0, contents.axis_headers[1].tile_padding(0));
    assert_eq!(0, contents.axis_headers[1].tile_padding(1));
    assert_eq!(0, contents.axis_headers[1].tile_padding(2));
    assert_eq!(0, contents.axis_headers[1].tile_padding(3));
    assert_eq!(63, contents.axis_headers[1].tile_padding(4));
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
                .as_2d()
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

    let mut tiles = contents.tiles();

    let assert_absolute_pos_range =
        |tiles: &mut Tiles, range_axis_1: Range<_>, range_axis_2: Range<_>| {
            let tile = tiles.next().unwrap();

            for (pos, _) in tile.iter_with_abolute_pos().as_2d() {
                assert!(range_axis_1.contains(&pos.0));
                assert!(range_axis_2.contains(&pos.1));
            }
        };

    assert_absolute_pos_range(&mut tiles, 0..128, 0..64);
    assert_absolute_pos_range(&mut tiles, 0..128, 64..128);
    assert_absolute_pos_range(&mut tiles, 0..128, 128..192);
    assert_absolute_pos_range(&mut tiles, 0..128, 192..256);
    assert_absolute_pos_range(&mut tiles, 0..128, 256..257);
}

#[test]
fn correct_tiles_padding() {
    let contents = include_bytes!("./data/Nhsqc_highres_600MHz.ucsf");

    let (_, contents) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    let mut tiles = contents.tiles();

    let assert_absolute_pos = |tiles: &mut Tiles, padding| {
        let tile = tiles.next().unwrap();
        let tile_padding = (tile.axis_lengths[0], tile.axis_lengths[1]);
        assert_eq!(padding, tile_padding);
    };

    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 1));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 1));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 1));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 64));
    assert_absolute_pos(&mut tiles, (128, 1));
}
