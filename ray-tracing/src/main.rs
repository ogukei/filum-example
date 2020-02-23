
#[macro_use]
extern crate filum;
use filum::{Context, BufferViewBuilder, PipelineBuilder, DispatchBuilder};

extern crate image;

extern crate rand;

use rand::prelude::*;

use std::collections::HashMap;
use std::time::Instant;

#[repr(C)]
struct Sphere {
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
    mat: u32,
    reserved0: u32,
    reserved1: u32,
    reserved2: u32,
}

#[repr(C)]
struct Material {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    ty: u32,
    reserved0: u32,
    reserved1: u32,
    reserved2: u32,
}

fn distance(ax: f32, ay: f32, az: f32, bx: f32, by: f32, bz: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let dz = bz - az;
    return (dx * dx + dy * dy + dz * dz).sqrt();
}

// Ray Tracing in One Weekend
// Copyright 2018-2019. Peter Shirley. All rights reserved.
// @see https://raytracing.github.io/books/RayTracingInOneWeekend.html
fn main() {
    // opens image file
    println!("processing input image");
    let mut img_view = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(400, 200);
    let (width, height) = img_view.dimensions();
    let (width, height) = (width as usize, height as usize);
    let len = width * height;
    println!("shader setup");
    let num_objects = 30 + 4;
    // setup some shaders
    let context = Context::new().unwrap();
    let view = BufferViewBuilder::new(&context)
        .layout(
            bindings!(
                binding_array!(f32, len * 4),
                binding_array!(Sphere, num_objects),
                binding_array!(Material, num_objects),
            )
        )
        .build()
        .unwrap();
    let buffer = view.buffer();
    let first = PipelineBuilder::new(buffer)
        .shader("data/ray.comp.spv")
        .specialization(constants!(width as u32, height as u32, num_objects as u32))
        .build()
        .unwrap();
    println!("uploading");
    let mut rng = rand::thread_rng();
    let instant = Instant::now();
    let binding_image = view.first_binding();
    let binding_objects = view.second_binding();
    let binding_materials = view.third_binding();
    let from_x = -2isize;
    let to_x = 4isize;
    let from_y = 2isize;
    let to_y = 7isize;
    binding_objects.update_array(|slice| {
        {
            let object = &mut slice[0];
            object.x = 0.0;
            object.y = -1000.0;
            object.z = 0.0;
            object.radius = 1000.0;
            object.mat = 0;
        }
        {
            let object = &mut slice[1];
            object.x = 0.0;
            object.y = 1.0;
            object.z = 0.0;
            object.radius = 1.0;
            object.mat = 1;
        }
        {
            let object = &mut slice[2];
            object.x = -4.0;
            object.y = 1.0;
            object.z = 0.0;
            object.radius = 1.0;
            object.mat = 2;
        }
        {
            let object = &mut slice[3];
            object.x = 4.0;
            object.y = 1.0;
            object.z = 0.0;
            object.radius = 1.0;
            object.mat = 3;
        }
        let ranges = (from_y..to_y)
            .flat_map(|v| (from_x..to_x).map(move |w| (v, w)))
            .map(|(a, b)| (a as f32, b as f32));
        let it = slice.iter_mut()
            .enumerate()
            .skip(4);
        for (it, range) in it.zip(ranges) {
            let (index, object) = it;
            let (a, b) = range;
            let x = a + 0.9 * rng.gen::<f32>();
            let y = 0.2;
            let z = b + 0.9 * rng.gen::<f32>();
            if distance(x, y, z, 4.0, 0.2, 0.0) > 0.9 {
                object.x = x;
                object.y = y;
                object.z = z;
                object.radius = 0.2;
                object.mat = index as u32;
            }
        }
    });
    binding_materials.update_array(|slice| {
        {
            let material = &mut slice[0];
            material.ty = 0;
            material.x = 0.5;
            material.y = 0.5;
            material.z = 0.5;
        }
        {
            let material = &mut slice[1];
            material.ty = 2;
            material.w = 1.5;
        }
        {
            let material = &mut slice[2];
            material.ty = 0;
            material.x = 0.4;
            material.y = 0.2;
            material.z = 0.1;
        }
        {
            let material = &mut slice[3];
            material.ty = 1;
            material.x = 0.7;
            material.y = 0.6;
            material.z = 0.5;
            material.w = 0.0;
        }
        let ranges = (from_y..to_y)
            .flat_map(|v| (from_x..to_x).map(move |w| (v, w)))
            .map(|(a, b)| (a as f32, b as f32));
        let it = slice.iter_mut()
            .enumerate()
            .skip(4);
        for (it, _) in it.zip(ranges) {
            let (_, material) = it;
            let choose_mat = rng.gen::<f32>();
            if choose_mat < 0.8 {
                material.ty = 0;
                material.x = rng.gen::<f32>() * rng.gen::<f32>();
                material.y = rng.gen::<f32>() * rng.gen::<f32>();
                material.z = rng.gen::<f32>() * rng.gen::<f32>();
            } else if choose_mat < 0.95 {
                material.ty = 1;
                material.x = 0.5 * (1.0 + rng.gen::<f32>());
                material.y = 0.5 * (1.0 + rng.gen::<f32>());
                material.z = 0.5 * (1.0 + rng.gen::<f32>());
                material.z = 0.5 * rng.gen::<f32>();
            } else {
                material.ty = 2;
                material.w = 1.5;
            }
        }
    });
    println!("dispatch");
    let dispatch = DispatchBuilder::new(&first)
        .workgroup_count(width, height, 1)
        .build()
        .unwrap();
    dispatch.dispatch();
    println!("fetching");
    binding_image.fetch_array(|slice| {
        println!("done {:?}", instant.elapsed());
        output(slice, &mut img_view);
    });
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
