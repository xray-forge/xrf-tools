use crate::pass::camera_uniform::CameraUniform;

/// One viewport's camera on the GPU: its uniform buffer and the bind group passes read it through.
pub struct ViewBinding {
  pub buffer: wgpu::Buffer,
  pub bind_group: wgpu::BindGroup,
}

impl ViewBinding {
  /// The layout every pass binds a viewport's camera with, at group zero.
  pub fn create_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("view"),
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
    })
  }

  pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
    let buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("view camera"),
      size: size_of::<CameraUniform>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("view"),
      layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: buffer.as_entire_binding(),
      }],
    });

    Self { buffer, bind_group }
  }

  pub fn write(&self, queue: &wgpu::Queue, uniform: &CameraUniform) {
    queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(uniform));
  }
}
