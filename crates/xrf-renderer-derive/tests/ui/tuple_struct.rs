use xrf_renderer_core::ShaderStruct;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
struct Pair(f32, f32);

fn main() {}
