use ndarray::prelude::*;
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
    let arr_data = contents.data_continous();
    let arr = ArrayView::from_shape(contents.axis_sizes(), &arr_data).unwrap();

    let mut sorted_data = contents.data.to_vec();
    sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min_val: f32 = *sorted_data.first().unwrap();
    let max_val: f32 = *sorted_data.last().unwrap();

    let gradient = Gradient::with_domain(vec![
        (min_val, Srgb::<f32>::from_format(WHITE).into_linear()),
        (max_val, Srgb::<f32>::from_format(BLACK).into_linear()),
    ]);

    for (i_slice, slice) in arr.axis_iter(Axis(0)).enumerate() {
        let path = format!("examples/output/3d_{:04}.png", i_slice);
        let area = BitMapBackend::new(
            &path,
            (
                slice.axes().nth(0).unwrap().len() as u32,
                slice.axes().nth(1).unwrap().len() as u32,
            ),
        )
        .into_drawing_area();
        for (sub_dims, value) in slice.indexed_iter() {
            let i_axis_1 = sub_dims[0];
            let i_axis_2 = sub_dims[1];

            area.draw_pixel(
                (i_axis_1 as i32, i_axis_2 as i32),
                &gradient.get(*value).to_rgba(),
            )
            .unwrap();
        }
    }
}
