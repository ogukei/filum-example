
#[macro_use]
extern crate filum;
use filum::{Context, BufferViewBuilder, PipelineBuilder, DispatchBuilder};

extern crate image;

use std::collections::HashMap;
use std::time::Instant;

// This is actually an implementation of the following algorithm.
// A Parallel Approach to Object Identification in Large-scale Images
// @see https://www.academia.edu/29842500/
fn main() {
    // opens image file
    println!("processing input image");
    let mut img_view = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(2000, 1000);
    let (width, height) = img_view.dimensions();
    let (width, height) = (width as usize, height as usize);
    let len = width * height;
    println!("shader setup");
    // setup some shaders
    let context = Context::new().unwrap();
    let view = BufferViewBuilder::new(&context)
        .bind_array::<f32>(len * 4)
        .build()
        .unwrap();
    let buffer = view.buffer();
    let first = PipelineBuilder::new(buffer)
        .shader("data/first.comp.spv")
        .specialization(constants!(width as u32, height as u32))
        .build()
        .unwrap();
    println!("uploading");
    let mut instant = Instant::now();
    let binding = view.binding();
    binding.update_array(|slice| {
        input(slice, &mut img_view);
        instant = Instant::now();
    });
    println!("dispatch");
    let dispatch = DispatchBuilder::new(&first)
        .workgroup_count(width, height, 1)
        .build()
        .unwrap();
    dispatch.dispatch();
    println!("fetching");
    binding.fetch_array(|slice| {
        println!("done {:?}", instant.elapsed());
        output(slice, &mut img_view);
    });
}

fn input(slice: &mut [f32], img_view: &mut image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>) {

}

fn output(slice: &[f32], img_view: &mut image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>) {
    let width = img_view.width() as usize;
    let height = img_view.height() as usize;
    for (ix, iy, pixel) in img_view.enumerate_pixels_mut() {
        let (ix, iy) = (ix as usize, iy as usize);
        let iy = height - 1 - iy;
        let index = iy * width + ix;
        let rgba = &slice[index*4..(index*4)+4];
        let rgba = rgba.iter()
            .map(|&v| (v * 255.99) as u8)
            .collect::<Vec<u8>>();
        *pixel = image::Rgba([rgba[0], rgba[1], rgba[2], rgba[3]]);
    }
    img_view.save("output.png")
        .unwrap();
    println!("image saved as output.png");
}
