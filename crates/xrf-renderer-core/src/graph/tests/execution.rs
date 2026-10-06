use std::sync::atomic::{AtomicUsize, Ordering};

use crate::graph::tests::fixtures::{COLOR, color_texture, load};
use crate::graph::{
  FrameGraph, GraphBindings, GraphBufferAccess, GraphBufferDescriptor, GraphColorAttachment, GraphCompileOptions,
  GraphRuntime, GraphTextureAccess,
};
use crate::tests::test_device::create_device;

const SIZE: u32 = 64;
const BYTES: u64 = (SIZE * SIZE * 4) as u64;

/// Clears a transient red, draws over it in a pass sharing its render pass, and copies it into an imported buffer.
fn declare<'a>(graph: &mut FrameGraph<'a>, drawn: &'a AtomicUsize) -> crate::graph::GraphBuffer {
  let scene = graph.create_texture(color_texture("scene"));
  let output = graph.import_buffer(GraphBufferDescriptor::new("output", BYTES));

  graph
    .add_raster_pass("clear")
    .color(GraphColorAttachment::new(scene, wgpu::LoadOp::Clear(wgpu::Color::RED)))
    .record(move |_| {
      drawn.fetch_add(1, Ordering::Relaxed);
    });
  graph.add_raster_pass("over").color(load(scene)).record(move |_| {
    drawn.fetch_add(1, Ordering::Relaxed);
  });
  graph
    .add_encoder_pass("copy")
    .texture(scene, GraphTextureAccess::CopySource)
    .buffer(output, GraphBufferAccess::CopyDestination)
    .record(move |context| {
      let texture: &wgpu::Texture = context.get_texture(scene).texture;
      let buffer: &wgpu::Buffer = context.get_buffer(output);

      context.get_encoder().copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
          buffer,
          layout: wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(SIZE * 4),
            rows_per_image: Some(SIZE),
          },
        },
        wgpu::Extent3d {
          width: SIZE,
          height: SIZE,
          depth_or_array_layers: 1,
        },
      );
    });

  output
}

#[test]
fn executes_a_graph_and_reads_back_what_it_drew() {
  let Some((device, queue)) = create_device() else {
    return;
  };
  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("readback"),
    size: BYTES,
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });
  let mut runtime: GraphRuntime = GraphRuntime::new(&device, &queue);
  let drawn: AtomicUsize = AtomicUsize::new(0);

  for _ in 0..2 {
    let mut graph: FrameGraph<'_> = FrameGraph::new();
    let output = declare(&mut graph, &drawn);
    let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();

    assert_eq!(compiled.get_render_pass_count(), 1);

    let mut bindings: GraphBindings<'_> = GraphBindings::new();

    bindings.bind_buffer(output, &readback);
    queue.submit(compiled.execute(&device, &mut runtime, &bindings).unwrap().commands);
  }

  assert_eq!(drawn.load(Ordering::Relaxed), 4);
  assert_eq!(
    runtime.pool.get_texture_count(),
    1,
    "the second frame reuses the first frame's texture"
  );

  readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  let texels: Vec<u8> = readback.slice(..).get_mapped_range().unwrap().to_vec();

  assert_eq!(&texels[..4], &[255, 0, 0, 255]);
  assert_eq!(&texels[texels.len() - 4..], &[255, 0, 0, 255]);
}

#[test]
fn refuses_an_unbound_import() {
  let Some((device, queue)) = create_device() else {
    return;
  };
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let drawn: AtomicUsize = AtomicUsize::new(0);

  declare(&mut graph, &drawn);

  let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();
  let error: String = match compiled.execute(&device, &mut GraphRuntime::new(&device, &queue), &GraphBindings::new()) {
    Ok(_) => panic!("expected the unbound import to be refused"),
    Err(error) => error.to_string(),
  };

  assert!(error.contains("'output' is not bound"), "{error}");
}

#[test]
fn refuses_an_import_bound_to_a_mismatched_texture() {
  let Some((device, queue)) = create_device() else {
    return;
  };
  let small: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
    label: Some("small"),
    size: wgpu::Extent3d {
      width: 8,
      height: 8,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: COLOR,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    view_formats: &[],
  });
  let view: wgpu::TextureView = small.create_view(&wgpu::TextureViewDescriptor::default());
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let target = graph.import_texture(color_texture("target"));

  graph
    .add_raster_pass("clear")
    .color(GraphColorAttachment::new(target, wgpu::LoadOp::Clear(wgpu::Color::RED)))
    .record(|_| {});

  let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();
  let mut bindings: GraphBindings<'_> = GraphBindings::new();

  bindings.bind_texture(
    target,
    crate::graph::GraphResolvedTexture {
      texture: &small,
      view: &view,
    },
  );

  assert!(
    compiled
      .execute(&device, &mut GraphRuntime::new(&device, &queue), &bindings)
      .is_err()
  );
}

#[test]
fn times_each_pass_and_render_pass_on_the_gpu() {
  let Some((device, queue)) = create_device() else {
    return;
  };
  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("readback"),
    size: BYTES,
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });
  let mut runtime: GraphRuntime = GraphRuntime::new(&device, &queue);
  let drawn: AtomicUsize = AtomicUsize::new(0);

  runtime.timer.set_enabled(true);

  if !runtime.timer.is_timing() {
    eprintln!("Skipped: the device writes no timestamps inside encoders");

    return;
  }

  for _ in 0..4 {
    let mut graph: FrameGraph<'_> = FrameGraph::new();
    let output = declare(&mut graph, &drawn);
    let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();
    let mut bindings: GraphBindings<'_> = GraphBindings::new();

    bindings.bind_buffer(output, &readback);

    let executed = compiled.execute(&device, &mut runtime, &bindings).unwrap();

    assert_eq!(executed.groups.len(), 1);
    queue.submit(executed.commands);
    runtime.timer.request();
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
  }

  let names: Vec<String> = runtime.timer.take().into_iter().map(|time| time.name).collect();

  assert_eq!(names, ["clear + over", "copy"]);
}
