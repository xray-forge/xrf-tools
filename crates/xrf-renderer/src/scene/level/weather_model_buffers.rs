use wgpu::util::DeviceExt;

use crate::host::render_weather_model::RenderWeatherModel;

/// A model the weather draws, on the GPU: rain's splash, which every landing drop draws a copy of, or a thunderbolt. Its
/// vertices in renderer space, a position then a coordinate each, and its indices.
pub struct WeatherModelBuffers {
  pub vertices: wgpu::Buffer,
  pub indices: wgpu::Buffer,
  pub index_count: u32,
}

impl WeatherModelBuffers {
  /// The model as the weather draws it; an empty one for none, which draws nothing.
  pub fn new(device: &wgpu::Device, model: Option<&RenderWeatherModel>) -> Self {
    // A model naming a vertex it does not hold draws nothing.
    let model: Option<&RenderWeatherModel> = model.filter(|model| {
      let count: usize = (model.positions.len() / 3).min(model.uvs.len() / 2);

      model.indices.iter().all(|index| (*index as usize) < count)
    });
    let (vertices, indices): (Vec<[f32; 4]>, Vec<u32>) = model.map_or((Vec::new(), Vec::new()), |model| {
      let count: usize = (model.positions.len() / 3).min(model.uvs.len() / 2);
      let vertices = (0..count)
        .flat_map(|vertex| {
          let [x, y, z] = [0, 1, 2].map(|axis| model.positions[vertex * 3 + axis]);

          // Engine `z` negated into renderer space.
          [
            [x, y, -z, 1.0],
            [model.uvs[vertex * 2], model.uvs[vertex * 2 + 1], 0.0, 0.0],
          ]
        })
        .collect();
      let indices = model.indices.iter().map(|index| *index as u32).collect();

      (vertices, indices)
    });
    let index_count: u32 = (indices.len() / 3 * 3) as u32;
    let buffer = |label: &str, contents: &[u8]| {
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: if contents.is_empty() { &[0; 16] } else { contents },
        usage: wgpu::BufferUsages::STORAGE,
      })
    };

    Self {
      vertices: buffer("weather model vertices", bytemuck::cast_slice(&vertices)),
      indices: buffer("weather model indices", bytemuck::cast_slice(&indices)),
      index_count,
    }
  }
}
