use std::sync::Mutex;

use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{BvhTracer, Tracer};
use xrf_renderer_core::RayQueryDevice;
use xrf_renderer_core::wgpu;
use xrf_renderer_core::wgpu::util::DeviceExt;

use crate::detail::gpu::detail_gpu_batch::DetailGpuBatch;
use crate::detail::gpu::detail_gpu_records::{GpuAlphaFace, GpuAlphaTexture, GpuBatchSize};
use crate::shadow::shadow_materials::ShadowMaterials;

/// Traces the detail lighter's shadow rays on the GPU with ray queries (`detail_shadow_rays.wgsl`): the ray model as one
/// acceleration structure of two geometries, its opaque faces and its alpha faces, and the alpha faces' textures
/// uploaded once; then each batch of slots' rays made from their plans and answered with how much light each lets
/// through, as `DetailSlotLighter::trace` answers it.
pub(crate) struct DetailGpuTracer {
  device: RayQueryDevice,
  packets_pipeline: wgpu::ComputePipeline,
  singles_pipeline: wgpu::ComputePipeline,
  /// What every batch binds alike: the structure, the alpha faces and textures, the batch's size.
  fixed: DetailGpuFixed,
  /// The batch's buffers and the bind group over them, made again larger when a batch outgrows them.
  batch: Mutex<Option<DetailGpuBatchBuffers>>,
  /// Held for the top level, which names it.
  _blas: wgpu::Blas,
}

/// The buffers bound for every batch.
struct DetailGpuFixed {
  tlas: wgpu::Tlas,
  alpha_faces: wgpu::Buffer,
  alpha_textures: wgpu::Buffer,
  alpha: wgpu::Buffer,
  size: wgpu::Buffer,
}

/// A batch's buffers, each holding `capacity` of its records, and the bind group over them.
struct DetailGpuBatchBuffers {
  /// Packets, origins, singles and rays each holds.
  capacity: [usize; 4],
  packets: wgpu::Buffer,
  origins: wgpu::Buffer,
  singles: wgpu::Buffer,
  transmittances: wgpu::Buffer,
  readback: wgpu::Buffer,
  bind_group: wgpu::BindGroup,
}

impl DetailGpuTracer {
  /// Lanes a single rays' workgroup runs, `SINGLE_LANES` in the shader; a packet takes a workgroup of its own.
  const SINGLE_LANES: u32 = 64;

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
    let [opaque, alpha_corners]: [Vec<[f32; 3]>; 2] = Self::split_faces(tracer, shadows)?;
    let storage = |label: &str, bytes: &[u8], usage: wgpu::BufferUsages| -> wgpu::Buffer {
      // A binding cannot be empty: an empty array is bound as one element of the widest record.
      let padded: Vec<u8> = if bytes.is_empty() {
        vec![0; size_of::<GpuAlphaFace>()]
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
          "detail ray model",
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
    let alpha_faces: Vec<GpuAlphaFace> = shadows.get_alpha_faces().map(GpuAlphaFace::of).collect();
    let (alpha_textures, alpha): (Vec<GpuAlphaTexture>, Vec<u8>) = GpuAlphaTexture::pack(shadows.get_textures());
    let blas: wgpu::Blas = gpu.create_blas(
      &wgpu::CreateBlasDescriptor {
        label: Some("detail ray model"),
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
      },
      wgpu::BlasGeometrySizeDescriptors::Triangles {
        descriptors: geometries.iter().map(|(_, size)| size.clone()).collect(),
      },
    );
    let mut tlas: wgpu::Tlas = gpu.create_tlas(&wgpu::CreateTlasDescriptor {
      label: Some("detail ray model"),
      max_instances: 1,
      flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
      update_mode: wgpu::AccelerationStructureUpdateMode::Build,
    });

    tlas[0] = Some(wgpu::TlasInstance::new(&blas, Self::IDENTITY, 0, 0xff));

    let mut encoder: wgpu::CommandEncoder = gpu.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("detail ray model build"),
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
      label: Some("detail shadow rays"),
      source: wgpu::ShaderSource::Wgsl(include_str!("detail_shadow_rays.wgsl").into()),
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
      label: Some("detail shadow rays"),
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
      label: Some("detail shadow rays"),
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
    let fixed: DetailGpuFixed = DetailGpuFixed {
      tlas,
      alpha_faces: storage(
        "detail alpha faces",
        bytemuck::cast_slice(&alpha_faces),
        wgpu::BufferUsages::empty(),
      ),
      alpha_textures: storage(
        "detail alpha textures",
        bytemuck::cast_slice(&alpha_textures),
        wgpu::BufferUsages::empty(),
      ),
      alpha: storage("detail alpha", &alpha, wgpu::BufferUsages::empty()),
      size: gpu.create_buffer(&wgpu::BufferDescriptor {
        label: Some("detail batch size"),
        size: size_of::<GpuBatchSize>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
    };

    if let Some(error) = pollster::block_on(scope.pop()) {
      return Err(XrfError::new_invalid_error(format!(
        "The GPU '{}' refused the detail tracer: {error}",
        device.name
      )));
    }

    Ok(Self {
      device,
      packets_pipeline,
      singles_pipeline,
      fixed,
      batch: Mutex::new(None),
      _blas: blas,
    })
  }

  /// The name of the GPU tracing.
  pub(crate) fn get_name(&self) -> &str {
    &self.device.name
  }

  /// How much light each of `batch`'s rays lets through, by its number.
  ///
  /// # Errors
  ///
  /// Returns an error when the batch holds more packets or single rays than one dispatch takes, the GPU refuses its
  /// buffers or dispatches, or its answers cannot be read back.
  pub(crate) fn trace(&self, batch: &DetailGpuBatch) -> XrfResult<Vec<f32>> {
    let packets: usize = batch.packets.len();
    let singles: usize = batch.singles.len();

    if packets > Self::MOST_WORKGROUPS || singles > Self::MOST_SINGLES {
      return Err(XrfError::new_invalid_error(format!(
        "A detail batch of {packets} packets and {singles} single rays is more than one dispatch takes"
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
      .map_err(|_| XrfError::new_unexpected_error("A detail batch failed while holding the GPU's buffers"))?;
    let wanted: [usize; 4] = [packets, batch.origins.len(), singles, batch.ray_count];

    if buffers
      .as_ref()
      .is_some_and(|held| held.capacity.iter().zip(wanted).any(|(held, wanted)| *held < wanted))
    {
      *buffers = None;
    }

    let held: &DetailGpuBatchBuffers = buffers.get_or_insert_with(|| self.create_batch_buffers(wanted));
    let bytes: u64 = (batch.ray_count * size_of::<f32>()) as u64;

    queue.write_buffer(&held.packets, 0, bytemuck::cast_slice(&batch.packets));
    queue.write_buffer(&held.origins, 0, bytemuck::cast_slice(&batch.origins));
    queue.write_buffer(&held.singles, 0, bytemuck::cast_slice(&batch.singles));
    queue.write_buffer(
      &self.fixed.size,
      0,
      bytemuck::bytes_of(&GpuBatchSize::new(packets as u32, singles as u32)),
    );

    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("detail shadow rays"),
    });

    {
      let mut pass: wgpu::ComputePass<'_> = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("detail shadow rays"),
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
        "The GPU '{}' refused the detail shadow rays: {error}",
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
  fn create_batch_buffers(&self, wanted: [usize; 4]) -> DetailGpuBatchBuffers {
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
    let packets: wgpu::Buffer = buffer("detail packets", capacity[0] * 64, input);
    let origins: wgpu::Buffer = buffer("detail origins", capacity[1] * 16, input);
    let singles: wgpu::Buffer = buffer("detail singles", capacity[2] * 32, input);
    let transmittances: wgpu::Buffer = buffer(
      "detail transmittances",
      capacity[3] * 4,
      wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let readback: wgpu::Buffer = buffer(
      "detail transmittances read back",
      capacity[3] * 4,
      wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    );
    let entry = Self::entry;
    let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("detail shadow rays"),
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
      ],
    });

    DetailGpuBatchBuffers {
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
  /// triangle, its corners not numbers, which no ray meets, since a structure's geometry cannot be empty.
  fn split_faces(tracer: &BvhTracer, shadows: &ShadowMaterials) -> XrfResult<[Vec<[f32; 3]>; 2]> {
    let codes: Vec<u32> = shadows.get_face_codes();
    let alpha_count: usize = shadows.get_alpha_faces().count();
    let mut opaque: Vec<[f32; 3]> = Vec::new();
    let mut alpha: Vec<[f32; 3]> = vec![[f32::NAN; 3]; alpha_count.max(1) * 3];

    for (face, code) in codes.iter().enumerate() {
      let face: u32 = u32::try_from(face)
        .map_err(|_| XrfError::new_invalid_error("The ray model holds more faces than a GPU structure takes"))?;
      let corners: [[f32; 3]; 3] = tracer.get_triangle(face).corners.map(|corner| corner.to_array());

      match *code {
        ShadowMaterials::FACE_PASSES => {}
        ShadowMaterials::FACE_OPAQUE => opaque.extend(corners),
        alpha_code => {
          let index: usize = (alpha_code - ShadowMaterials::FACE_ALPHA) as usize * 3;

          alpha[index..index + 3].copy_from_slice(&corners);
        }
      }
    }

    if opaque.is_empty() {
      opaque = vec![[f32::NAN; 3]; 3];
    }

    Ok([opaque, alpha])
  }
}
