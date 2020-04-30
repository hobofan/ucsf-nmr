use palette::Srgb;
use palette::{
    named::{BLACK, WHITE},
    Gradient,
};
use plotters::prelude::*;
use ucsf_nmr::UcsfFile;

pub fn main() {
    let contents = include_bytes!("../tests/data/15n_hsqc.ucsf");

    let (_, contents) = UcsfFile::parse(&contents[..]).expect("Failed parsing");

    let root = BitMapBackend::new(
        "examples/output/all_tiles.png",
        (
            contents.axis_data_points(0) as u32,
            contents.axis_data_points(1) as u32,
        ),
    )
    .into_drawing_area();

    let mut sorted_data = contents.data.to_vec();
    sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min_val: f32 = *sorted_data.first().unwrap();
    let max_val: f32 = *sorted_data.last().unwrap();

    let gradient = Gradient::with_domain(vec![
        (min_val, Srgb::<f32>::from_format(WHITE).into_linear()),
        (max_val, Srgb::<f32>::from_format(BLACK).into_linear()),
    ]);
    let data = contents.data_continous();
    let dimensions = contents.axis_sizes();
    for d1 in 0..dimensions[0] {
        for d2 in 0..dimensions[1] {
            let pos = d1 * dimensions[1] + d2;
            let value = data[pos];
            root.draw_pixel((d1 as i32, d2 as i32), &gradient.get(value).to_rgba())
                .unwrap();
        }
    }
}
