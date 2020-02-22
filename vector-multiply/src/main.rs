
#[macro_use]
extern crate filum;
use filum::{Context, BufferViewBuilder, PipelineBuilder};

fn main() {
    let num_elements = 128usize;
    let context = Context::new().unwrap();
    let buffer_view = BufferViewBuilder::new(&context)
        .layout(
            bindings!(
                binding_array!(f32, num_elements),
                binding_array!(f32, num_elements),
            )
        )
        .build()
        .unwrap();
    let buffer = buffer_view.buffer();
    let pipeline = PipelineBuilder::new(buffer)
        .shader("data/multiply.comp.spv")
        .build()
        .unwrap();
    let lhs = buffer_view.first_binding();
    let rhs = buffer_view.second_binding();
    lhs.update_array(|slice| {
        for (index, value) in slice.iter_mut().enumerate() {
            *value = index as f32;
        }
    });
    rhs.update_array(|slice| {
        for (index, value) in slice.iter_mut().enumerate() {
            *value = index as f32;
        }
    });
    pipeline.dispatch(num_elements);
    lhs.fetch_array(|slice| {
        println!("{:?}", slice);
    });
}
