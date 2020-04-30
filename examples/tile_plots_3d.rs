use palette::Srgb;
use palette::{
    named::{BLACK, WHITE},
    Gradient,
};
use plotters::prelude::*;
use ucsf_nmr::UcsfFile;

pub fn main() {
    let contents = std::fs::read("./tests/data/c13_noesy_aliph.ucsf").unwrap();

    let (_, contents) = UcsfFile::parse(&contents[..]).expect("Failed parsing");

    let drawing_area_size = (
        contents.axis_data_points(1) as u32,
        contents.axis_data_points(2) as u32,
    );
    let image_paths: Vec<_> = (0..contents.axis_data_points(0))
        .into_iter()
        .map(move |slice| std::path::PathBuf::from(format!("examples/output/3d_{:04}.png", slice)))
        .collect();
    let areas: Vec<_> = image_paths
        .iter()
        .map(|path| {
            let area = BitMapBackend::new(path, drawing_area_size).into_drawing_area();
            area
        })
        .collect();

    let mut sorted_data = contents.data.to_vec();
    sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min_val: f32 = *sorted_data.first().unwrap();
    let max_val: f32 = *sorted_data.last().unwrap();

    let gradient = Gradient::with_domain(vec![
        (min_val, Srgb::<f32>::from_format(WHITE).into_linear()),
        (max_val, Srgb::<f32>::from_format(BLACK).into_linear()),
    ]);
    for tile in contents.tiles() {
        for ((slice, i_axis_1, i_axis_2), value) in tile.iter_with_abolute_pos().as_3d() {
            areas[slice]
                .draw_pixel(
                    (i_axis_1 as i32, i_axis_2 as i32),
                    &gradient.get(value).to_rgba(),
                )
                .unwrap();
        }
    }
}
