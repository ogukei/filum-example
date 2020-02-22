
extern crate filum;
use filum::{Context, BufferViewBuilder, PipelineBuilder};

fn main() {
    fibonacci(32);
}

fn fibonacci(num_elements: usize) {
    // setup data to calculate fibonacci sequence
    let mut v: Vec<u32> = (0..num_elements as u32).collect();
    // filum automatically selects one from available GPUs. 
    // context contains information of the GPU.
    let context = Context::new().unwrap();
    // allocates contiguous memory of u32 with the number of `num_elements`.
    let buffer_view = BufferViewBuilder::new(&context)
        .bind_array::<u32>(num_elements)
        .build()
        .unwrap();
    // loads a compute shader from the specified file path
    // and associates it to the buffer.
    let pipeline = PipelineBuilder::new(buffer_view.buffer())
        .shader("data/fibonacci.comp.spv")
        .build()
        .unwrap();
    // in order to transfer our data to the GPU, 
    // gets a reference which corresponds to the binding point that
    // indicates the location of the array of u32 stored on the GPU.
    let binding = buffer_view.binding();
    // sends data to the GPU
    binding.update_array_copying(&v);
    // runs the computation specifying how many invocations of 
    // the shader performed.
    pipeline.dispatch(num_elements);
    // retrieves back data from the GPU
    binding.fetch_array_copying(&mut v);
    println!("{:?}", v);
}
