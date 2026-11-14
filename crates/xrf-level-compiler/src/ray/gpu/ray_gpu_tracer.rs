use std::sync::Mutex;

use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{BvhTracer, Tracer};
use xrf_renderer_core::RayQueryDevice;
use xrf_renderer_core::wgpu;
use xrf_renderer_core::wgpu::util::DeviceExt;

use crate::ray::gpu::ray_gpu_alpha_face::RayGpuAlphaFace;
use crate::ray::gpu::ray_gpu_alpha_texture::RayGpuAlphaTexture;
use crate::ray::gpu::ray_gpu_alpha_textures::RayGpuAlphaTextures;
use crate::ray::gpu::ray_gpu_batch::RayGpuBatch;
use crate::ray::gpu::ray_gpu_batch_size::RayGpuBatchSize;
use crate::ray::gpu::ray_gpu_origin::RayGpuOrigin;
use crate::ray::gpu::ray_gpu_packet::RayGpuPacket;
use crate::ray::gpu::ray_gpu_single::RayGpuSingle;
use crate::ray::ray_rules::RayRules;
use crate::shadow::shadow_face::ShadowFace;
use crate::shadow::shadow_materials::ShadowMaterials;

/// Traces shadow rays on the GPU with ray queries (`shadow_rays.wgsl`), for the detail lighter, the AI map's covers
/// and the lightmaps: the ray model as one acceleration structure of two geometries, its opaque faces and its alpha
/// faces, and the alpha faces' textures uploaded once; then each batch's rays made from their plans and answered with
/// how much light each lets through, as the CPU answers it by a batch's [`RayRules`]: which sides of a face stop a
/// ray, how much an alpha texel lets through, whether a ray passes a face of its own, and whether a face counts only
/// where the CPU's ray test meets it too.
pub(crate) struct RayGpuTracer {
  device: RayQueryDevice,
  packets_pipeline: wgpu::ComputePipeline,
  singles_pipeline: wgpu::ComputePipeline,
  /// What every batch binds alike: the structure, the alpha faces and textures, the batch's size.
  fixed: RayGpuFixed,
  /// The batch's buffers and the bind group over them, made again larger when a batch outgrows them.
  batch: Mutex<Option<RayGpuBatchBuffers>>,
  /// Each ray model face's skip code: its geometry in the top bit and its place there; [`Self::NO_SKIP`] for a face
  /// passing light, which the structure leaves out.
  skips: Vec<u32>,
  /// Held for the top level, which names it.
  _blas: wgpu::Blas,
}

/// The corners of the ray model's opaque faces and of its alpha faces, and each face's skip code.
struct RaySplitFaces {
  opaque: Vec<[f32; 3]>,
  alpha: Vec<[f32; 3]>,
  skips: Vec<u32>,
}

/// The buffers bound for every batch.
struct RayGpuFixed {
  tlas: wgpu::Tlas,
  /// The opaque geometry's corners and the alpha geometry's, the structure's inputs, read where a candidate is tested
  /// as the CPU tests it.
  corners: [wgpu::Buffer; 2],
  alpha_faces: wgpu::Buffer,
  alpha_textures: wgpu::Buffer,
  alpha: wgpu::Buffer,
  size: wgpu::Buffer,
}

/// A batch's buffers, each holding `capacity` of its records, and the bind group over them.
struct RayGpuBatchBuffers {
  /// Packets, origins, singles and rays each holds.
  capacity: [usize; 4],
  packets: wgpu::Buffer,
  origins: wgpu::Buffer,
  singles: wgpu::Buffer,
  transmittances: wgpu::Buffer,
  readback: wgpu::Buffer,
  bind_group: wgpu::BindGroup,
}

impl RayGpuTracer {
  /// Lanes a single rays' workgroup runs, `SINGLE_LANES` in the shader; a packet takes a workgroup of its own.
  const SINGLE_LANES: u32 = 64;

  /// The skip code of no face.
  pub(crate) const NO_SKIP: u32 = u32::MAX;

  /// The skip code's bit naming the alpha geometry.
  const ALPHA_GEOMETRY: u32 = 1 << 31;

  /// The most workgroups a dispatch's side holds.
  pub(crate) const MOST_WORKGROUPS: usize = 65_535;

  /// The most single rays one batch holds.
  pub(crate) const MOST_SINGLES: usize = Self::MOST_WORKGROUPS * Self::SINGLE_LANES as usize;

  /// The transform placing the ray model where it stands, a 3 by 4 row-major identity.
  const IDENTITY: [f32; 12] = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];

  /// The tracer over `tracer`'s faces and `shadows`' parts, on `device`.
  ///
  /// # Errors
  ///
  /// Returns an error when wgpu refuses a buffer, the structure, the shader or the pipelines.
  pub(crate) fn new(device: RayQueryDevice, tracer: &BvhTracer, shadows: &ShadowMaterials) -> XrfResult<Self> {
    let gpu: &wgpu::Device = &device.device;
    let scope: wgpu::ErrorScopeGuard = gpu.push_error_scope(wgpu::ErrorFilter::Validation);
    let RaySplitFaces {
      opaque,
      alpha: alpha_corners,
      skips,
    } = Self::split_faces(tracer, shadows)?;
    let storage = |label: &str, bytes: &[u8], usage: wgpu::BufferUsages| -> wgpu::Buffer {
      // A binding cannot be empty: an empty array is bound as one element of the widest record.
      let padded: Vec<u8> = if bytes.is_empty() {
        vec![0; size_of::<RayGpuAlphaFace>()]
      } else {
        bytes.to_vec()
      };

      gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: &padded,
        usage: wgpu::BufferUsages::STORAGE | usage,
      })
    };
    let geometries: [(wgpu::Buffer, wgpu::BlasTriangleGeometrySizeDescriptor); 2] = [
      (&opaque, wgpu::AccelerationStructureGeometryFlags::OPAQUE),
      // Each alpha face asked about once a ray, as the product needs.
      (
        &alpha_corners,
        wgpu::AccelerationStructureGeometryFlags::NO_DUPLICATE_ANY_HIT_INVOCATION,
      ),
    ]
    .map(|(corners, flags)| {
      (
        storage(
          "ray model",
          bytemuck::cast_slice(corners),
          wgpu::BufferUsages::BLAS_INPUT,
        ),
        wgpu::BlasTriangleGeometrySizeDescriptor {
          vertex_format: wgpu::VertexFormat::Float32x3,
          vertex_count: corners.len() as u32,
          index_format: None,
          index_count: None,
          flags,
        },
      )
    });
    let alpha_faces: Vec<RayGpuAlphaFace> = shadows.get_alpha_faces().map(RayGpuAlphaFace::of).collect();
    let RayGpuAlphaTextures {
      records: alpha_textures,
      bytes: alpha,
    } = RayGpuAlphaTexture::pack(shadows.get_textures());
    let blas: wgpu::Blas = gpu.create_blas(
      &wgpu::CreateBlasDescriptor {
        label: Some("ray model"),
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
      },
      wgpu::BlasGeometrySizeDescriptors::Triangles {
        descriptors: geometries.iter().map(|(_, size)| size.clone()).collect(),
      },
    );
    let mut tlas: wgpu::Tlas = gpu.create_tlas(&wgpu::CreateTlasDescriptor {
      label: Some("ray model"),
      max_instances: 1,
      flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
      update_mode: wgpu::AccelerationStructureUpdateMode::Build,
    });

    tlas[0] = Some(wgpu::TlasInstance::new(&blas, Self::IDENTITY, 0, 0xff));

    let mut encoder: wgpu::CommandEncoder = gpu.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("ray model build"),
    });

    encoder.build_acceleration_structures(
      std::iter::once(&wgpu::BlasBuildEntry {
        blas: &blas,
        geometry: wgpu::BlasGeometries::TriangleGeometries(
          geometries
            .iter()
            .map(|(vertices, size)| wgpu::BlasTriangleGeometry {
              size,
              vertex_buffer: vertices,
              first_vertex: 0,
              vertex_stride: 12,
              index_buffer: None,
              first_index: None,
              transform_buffer: None,
              transform_buffer_offset: None,
            })
            .collect(),
        ),
      }),
      std::iter::once(&tlas),
    );
    device.queue.submit(std::iter::once(encoder.finish()));

    let module: wgpu::ShaderModule = gpu.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("shadow rays"),
      source: wgpu::ShaderSource::Wgsl(include_str!("shadow_rays.wgsl").into()),
    });
    // One layout both entries share, as each alone names only the bindings it reads.
    let storage_entry = |binding: u32, read_only: bool| -> wgpu::BindGroupLayoutEntry {
      wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Storage { read_only },
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }
    };
    let layout: wgpu::BindGroupLayout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("shadow rays"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::COMPUTE,
          ty: wgpu::BindingType::AccelerationStructure { vertex_return: false },
          count: None,
        },
        storage_entry(1, true),
        storage_entry(2, true),
        storage_entry(3, true),
        storage_entry(4, true),
        storage_entry(5, true),
        storage_entry(6, true),
        storage_entry(7, false),
        storage_entry(9, true),
        storage_entry(10, true),
        wgpu::BindGroupLayoutEntry {
          binding: 8,
          visibility: wgpu::ShaderStages::COMPUTE,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
      ],
    });
    let pipeline_layout: wgpu::PipelineLayout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("shadow rays"),
      bind_group_layouts: &[Some(&layout)],
      immediate_size: 0,
    });
    let pipeline = |entry: &str| -> wgpu::ComputePipeline {
      gpu.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(entry),
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some(entry),
        compilation_options: Default::default(),
        cache: None,
      })
    };
    let packets_pipeline: wgpu::ComputePipeline = pipeline("cs_packets");
    let singles_pipeline: wgpu::ComputePipeline = pipeline("cs_singles");
    let [(opaque_corners, _), (alpha_corners, _)] = geometries;
    let fixed: RayGpuFixed = RayGpuFixed {
      tlas,
      corners: [opaque_corners, alpha_corners],
      alpha_faces: storage(
        "ray alpha faces",
        bytemuck::cast_slice(&alpha_faces),
        wgpu::BufferUsages::empty(),
      ),
      alpha_textures: storage(
        "ray alpha textures",
        bytemuck::cast_slice(&alpha_textures),
        wgpu::BufferUsages::empty(),
      ),
      alpha: storage("ray alpha", &alpha, wgpu::BufferUsages::empty()),
      size: gpu.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ray batch size"),
        size: size_of::<RayGpuBatchSize>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
    };

    if let Some(error) = pollster::block_on(scope.pop()) {
      let messages: Vec<String> = pollster::block_on(module.get_compilation_info())
        .messages
        .iter()
        .map(|message| format!("{:?}: {}", message.location, message.message))
        .collect();

      return Err(XrfError::new_invalid_error(format!(
        "The GPU '{}' refused the ray tracer: {error}{}",
        device.name,
        messages.join("; ")
      )));
    }

    Ok(Self {
      device,
      packets_pipeline,
      singles_pipeline,
      fixed,
      batch: Mutex::new(None),
      skips,
      _blas: blas,
    })
  }

  /// The name of the GPU tracing.
  pub(crate) fn get_name(&self) -> &str {
    &self.device.name
  }

  /// The skip code of ray model face `face`, [`Self::NO_SKIP`] for none or one the structure leaves out.
  pub(crate) fn get_skip(&self, face: Option<u32>) -> u32 {
    face
      .and_then(|face| self.skips.get(face as usize).copied())
      .unwrap_or(Self::NO_SKIP)
  }

  /// How much light each of `batch`'s rays lets through, by its number, by `rules`.
  ///
  /// # Errors
  ///
  /// Returns an error when the batch holds more packets or single rays than one dispatch takes, the GPU refuses its
  /// buffers or dispatches, or its answers cannot be read back.
  pub(crate) fn trace(&self, batch: &RayGpuBatch, rules: RayRules) -> XrfResult<Vec<f32>> {
    let packets: usize = batch.packets.len();
    let singles: usize = batch.singles.len();

    if packets > Self::MOST_WORKGROUPS || singles > Self::MOST_SINGLES {
      return Err(XrfError::new_invalid_error(format!(
        "A ray batch of {packets} packets and {singles} single rays is more than one dispatch takes"
      )));
    }

    if batch.ray_count == 0 {
      return Ok(Vec::new());
    }

    let device: &wgpu::Device = &self.device.device;
    let queue: &wgpu::Queue = &self.device.queue;
    let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let mut buffers = self
      .batch
      .lock()
      .map_err(|_| XrfError::new_unexpected_error("A ray batch failed while holding the GPU's buffers"))?;
    let wanted: [usize; 4] = [packets, batch.origins.len(), singles, batch.ray_count];

    if buffers
      .as_ref()
      .is_some_and(|held| held.capacity.iter().zip(wanted).any(|(held, wanted)| *held < wanted))
    {
      *buffers = None;
    }

    let held: &RayGpuBatchBuffers = buffers.get_or_insert_with(|| self.create_batch_buffers(wanted));
    let bytes: u64 = (batch.ray_count * size_of::<f32>()) as u64;

    queue.write_buffer(&held.packets, 0, bytemuck::cast_slice(&batch.packets));
    queue.write_buffer(&held.origins, 0, bytemuck::cast_slice(&batch.origins));
    queue.write_buffer(&held.singles, 0, bytemuck::cast_slice(&batch.singles));
    queue.write_buffer(
      &self.fixed.size,
      0,
      bytemuck::bytes_of(&RayGpuBatchSize::new(packets as u32, singles as u32, rules.get_bits())),
    );

    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("shadow rays"),
    });

    {
      let mut pass: wgpu::ComputePass<'_> = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("shadow rays"),
        timestamp_writes: None,
      });

      pass.set_bind_group(0, &held.bind_group, &[]);

      if packets > 0 {
        pass.set_pipeline(&self.packets_pipeline);
        pass.dispatch_workgroups(packets as u32, 1, 1);
      }

      if singles > 0 {
        pass.set_pipeline(&self.singles_pipeline);
        pass.dispatch_workgroups((singles as u32).div_ceil(Self::SINGLE_LANES), 1, 1);
      }
    }

    encoder.copy_buffer_to_buffer(&held.transmittances, 0, &held.readback, 0, bytes);
    queue.submit(std::iter::once(encoder.finish()));

    let slice: wgpu::BufferSlice<'_> = held.readback.slice(..bytes);

    slice.map_async(wgpu::MapMode::Read, |_| {});
    device
      .poll(wgpu::PollType::wait_indefinitely())
      .map_err(|error| XrfError::new_unexpected_error(format!("The GPU did not finish tracing: {error}")))?;

    if let Some(error) = pollster::block_on(scope.pop()) {
      return Err(XrfError::new_invalid_error(format!(
        "The GPU '{}' refused the shadow rays: {error}",
        self.device.name
      )));
    }

    let transmittances: Vec<f32> = {
      let mapped: wgpu::BufferView = slice
        .get_mapped_range()
        .map_err(|error| XrfError::new_unexpected_error(format!("The traced rays were not read back: {error}")))?;

      bytemuck::cast_slice(&mapped).to_vec()
    };

    held.readback.unmap();

    Ok(transmittances)
  }

  /// Buffers holding `wanted` packets, origins, single rays and rays, with room to grow by half, and their bind group.
  fn create_batch_buffers(&self, wanted: [usize; 4]) -> RayGpuBatchBuffers {
    let device: &wgpu::Device = &self.device.device;
    let capacity: [usize; 4] = wanted.map(|count| (count + count / 2).max(1));
    let buffer = |label: &str, size: usize, usage: wgpu::BufferUsages| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size as u64,
        usage,
        mapped_at_creation: false,
      })
    };
    let input = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
    let packets: wgpu::Buffer = buffer("ray packets", capacity[0] * size_of::<RayGpuPacket>(), input);
    let origins: wgpu::Buffer = buffer("ray origins", capacity[1] * size_of::<RayGpuOrigin>(), input);
    let singles: wgpu::Buffer = buffer("ray singles", capacity[2] * size_of::<RayGpuSingle>(), input);
    let transmittances: wgpu::Buffer = buffer(
      "ray transmittances",
      capacity[3] * 4,
      wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let readback: wgpu::Buffer = buffer(
      "ray transmittances read back",
      capacity[3] * 4,
      wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    );
    let entry = Self::entry;
    let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("shadow rays"),
      layout: &self.packets_pipeline.get_bind_group_layout(0),
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::AccelerationStructure(&self.fixed.tlas),
        },
        entry(1, &packets),
        entry(2, &origins),
        entry(3, &singles),
        entry(4, &self.fixed.alpha_faces),
        entry(5, &self.fixed.alpha_textures),
        entry(6, &self.fixed.alpha),
        entry(7, &transmittances),
        entry(8, &self.fixed.size),
        entry(9, &self.fixed.corners[0]),
        entry(10, &self.fixed.corners[1]),
      ],
    });

    RayGpuBatchBuffers {
      capacity,
      packets,
      origins,
      singles,
      transmittances,
      readback,
      bind_group,
    }
  }

  fn entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
      binding,
      resource: buffer.as_entire_binding(),
    }
  }

  /// The corners of the ray model's opaque faces, and of its alpha faces in their order among
  /// `ShadowMaterials::get_alpha_faces`; the faces passing light left out. A geometry left empty holds one inactive
  /// triangle, its corners not numbers, which no ray meets, since a structure's geometry cannot be empty. With them,
  /// each face's skip code, its place in its geometry.
  fn split_faces(tracer: &BvhTracer, shadows: &ShadowMaterials) -> XrfResult<RaySplitFaces> {
    let parts: &[ShadowFace] = shadows.get_faces();
    let alpha_count: usize = shadows.get_alpha_faces().count();
    let mut opaque: Vec<[f32; 3]> = Vec::new();
    let mut alpha: Vec<[f32; 3]> = vec![[f32::NAN; 3]; alpha_count.max(1) * 3];
    let mut skips: Vec<u32> = Vec::with_capacity(parts.len());

    for (face, part) in parts.iter().enumerate() {
      let face: u32 = u32::try_from(face)
        .map_err(|_| XrfError::new_invalid_error("The ray model holds more faces than a GPU structure takes"))?;
      let corners: [[f32; 3]; 3] = tracer.get_triangle(face).corners.map(|corner| corner.to_array());

      match *part {
        ShadowFace::Passes => skips.push(Self::NO_SKIP),
        ShadowFace::Opaque => {
          skips.push((opaque.len() / 3) as u32);
          opaque.extend(corners);
        }
        ShadowFace::Alpha(index) => {
          let start: usize = index as usize * 3;

          skips.push(Self::ALPHA_GEOMETRY | index);
          alpha[start..start + 3].copy_from_slice(&corners);
        }
      }
    }

    if opaque.is_empty() {
      opaque = vec![[f32::NAN; 3]; 3];
    }

    Ok(RaySplitFaces { opaque, alpha, skips })
  }
}
