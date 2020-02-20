
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
    let mut img = image::open("res/8k.png").unwrap();
    let img_view = img.as_mut_rgba8().unwrap();
    let (width, height) = img_view.dimensions();
    let (width, height) = (width as usize, height as usize);
    let len = width * height;
    assert!(width.is_power_of_two());
    println!("shader setup");
    // setup some shaders
    let context = Context::new().unwrap();
    let view = BufferViewBuilder::new(&context)
        .bind_array::<i32>(len)
        .build()
        .unwrap();
    let buffer = view.buffer();
    let column = PipelineBuilder::new(buffer)
        .shader("data/column.comp.spv")
        .specialization(constants!(width as u32, height as u32))
        .build()
        .unwrap();
    let merge = PipelineBuilder::new(buffer)
        .shader("data/merge.comp.spv")
        .specialization(constants!(width as u32, height as u32))
        .build()
        .unwrap();
    let relabel = PipelineBuilder::new(buffer)
        .shader("data/relabel.comp.spv")
        .build()
        .unwrap();
    println!("uploading");
    let mut instant = Instant::now();
    let binding = view.binding();
    binding.update_array(|slice| {
        input(slice, img_view);
        instant = Instant::now();
    });
    println!("column");
    column.dispatch(width);
    println!("merge");
    let mut step_index = 0;
    let mut n = width >> 1;
    while n != 0 {
        println!("n {}, si {}", n, step_index);
        let dispatch = DispatchBuilder::new(&merge)
            .workgroup_count(n, 1, 1)
            .push_constants(constants!(step_index as u32))
            .build()
            .unwrap();
        dispatch.dispatch();
        n = n >> 1;
        step_index += 1;
    }
    println!("relabel");
    relabel.dispatch(len);
    println!("fetching");
    binding.fetch_array(|slice| {
        println!("done {:?}", instant.elapsed());
        output(slice, img_view);
    });
}

fn input(slice: &mut [i32], img_view: &mut image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>) {
    let width = img_view.width();
    for (elem, pixel) in slice.iter_mut().zip(img_view.enumerate_pixels_mut()) {
        let (x, y, pixel) = pixel;
        let index = x + y * width;
        let image::Rgba(data) = *pixel;
        let alpha = data[3];
        if alpha != 0 {
            *elem = index as i32;
        } else {
            *elem = -1;
        }
    }
}

fn output(slice: &[i32], img_view: &mut image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>) {
    let palette = vec![
        0x580000,
        0xff4040,
        0xfd6a6a,
        0x001d72,
        0x0645ff,
        0x5681ff,
        0x004d02,
        0x09d20f,
        0x76ff7a,
        0x816102,
        0xffbf00,
        0xffd558,
        0x813f00,
        0xff8615,
        0xffbf83,
        0xffd7b2,
        0x130f0b,
        0x420056,
        0x8a00b4,
        0xde74ff,
        0xbababa,
        0x7a7a7a,
        0x4b4b4b,
    ]
        .iter()
        .map(|v| image::Rgba([(v >> 16) as u8, (v >> 8) as u8, (v >> 0) as u8,255]))
        .collect::<Vec<_>>();
    println!("processing output image");
    let width = img_view.width() as usize;
    let mut color_index = 0;
    let mut colors: HashMap<i32, image::Rgba<u8>> = HashMap::new();
    for (x, y, pixel) in img_view.enumerate_pixels_mut() {
        let index = (x + y * width as u32) as usize;
        let label = slice[index];
        if label >= 0 {
            if let Some(color) = colors.get(&label) {
                *pixel = *color;
            } else {
                let color = palette[color_index % palette.len()];
                colors.insert(label, color);
                color_index += 1;
            }
        }
    }
    img_view.save("output.png").unwrap();
    println!("image saved as output.png");
}
