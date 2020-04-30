use ucsf_nmr::UcsfFile;

#[test]
#[ignore]
fn parse_3d_file() {
    let contents = std::fs::read("./tests/data/c13_noesy_aliph.ucsf").unwrap();

    let (rem, _) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    assert_eq!(rem.len(), 0);
}
