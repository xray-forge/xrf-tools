use crate::alloc::{Span, SpanBuffer, UploadRing, UploadSlice};
use crate::tests::test_device::create_device;

/// Copies `buffer` into a mappable one and reads it back.
fn read_back(device: &wgpu::Device, queue: &wgpu::Queue, buffer: &wgpu::Buffer) -> Vec<u8> {
  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("readback"),
    size: buffer.size(),
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });
  let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

  encoder.copy_buffer_to_buffer(buffer, 0, &readback, 0, buffer.size());
  queue.submit([encoder.finish()]);
  readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  readback.slice(..).get_mapped_range().unwrap().to_vec()
}

#[test]
fn grows_a_span_buffer_keeping_what_it_held() {
  let Some((device, queue)) = create_device() else {
    return;
  };
  let mut buffer: SpanBuffer = SpanBuffer::new(&device, "test", 4, wgpu::BufferUsages::STORAGE, 4);
  let held: Span = buffer.allocate(&device, &queue, 4).unwrap();

  buffer.write(&held, 0, bytemuck::cast_slice(&[1u32, 2, 3, 4]));
  buffer.flush(&queue);

  let added: Span = buffer.allocate(&device, &queue, 4).unwrap();

  assert_eq!(buffer.get_generation(), 1);
  assert_eq!(buffer.get_capacity(), 12);

  buffer.write(&added, 0, bytemuck::cast_slice(&[5u32, 6, 7, 8]));
  buffer.write(&held, 3, bytemuck::cast_slice(&[40u32]));
  buffer.flush(&queue);

  let words: Vec<u32> = bytemuck::cast_slice(&read_back(&device, &queue, buffer.get_buffer())).to_vec();

  assert_eq!(words[..8], [1, 2, 3, 40, 5, 6, 7, 8]);
}

#[test]
fn uploads_a_frame_of_values_at_aligned_offsets() {
  let Some((device, queue)) = create_device() else {
    return;
  };
  let mut ring: UploadRing = UploadRing::new(
    &device,
    "test",
    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
    256,
  );
  let first: UploadSlice = ring.push(&[1.0f32, 2.0, 3.0, 4.0]);
  let second: UploadSlice = ring.push(&7u32);
  let alignment: u32 = ring.get_alignment();

  assert_eq!(first, UploadSlice { offset: 0, size: 16 });
  assert_eq!(
    second,
    UploadSlice {
      offset: alignment,
      size: 4
    }
  );

  ring.flush(&device, &queue);

  let bytes: Vec<u8> = read_back(&device, &queue, ring.get_buffer());

  assert_eq!(
    bytemuck::pod_read_unaligned::<[f32; 4]>(&bytes[..16]),
    [1.0, 2.0, 3.0, 4.0]
  );
  assert_eq!(
    bytemuck::pod_read_unaligned::<u32>(&bytes[alignment as usize..alignment as usize + 4]),
    7
  );
  assert_eq!(ring.get_pushed(), 0, "a flush starts the next frame's pushing");
}

#[test]
fn grows_an_upload_ring_that_a_frame_outgrows() {
  let Some((device, queue)) = create_device() else {
    return;
  };
  let mut ring: UploadRing = UploadRing::new(&device, "test", wgpu::BufferUsages::UNIFORM, 256);

  for value in 0..8u32 {
    ring.push(&value);
  }

  ring.flush(&device, &queue);

  assert_eq!(ring.get_generation(), 1);
  assert!(ring.get_buffer().size() >= u64::from(ring.get_alignment()) * 7 + 4);
}
