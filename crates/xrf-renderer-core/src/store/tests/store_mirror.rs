use crate::store::{ProxyHandle, ProxyStore, StoreMirror};
use crate::tests::test_device::create_device;

/// Reads a mirror's first `count` records back.
fn read_back(device: &wgpu::Device, queue: &wgpu::Queue, mirror: &StoreMirror, count: u32) -> Vec<u32> {
  let size: u64 = u64::from(count) * 4;
  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("readback"),
    size,
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });
  let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

  encoder.copy_buffer_to_buffer(mirror.get_buffer(), 0, &readback, 0, size);
  queue.submit([encoder.finish()]);
  readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  let words: Vec<u32> = bytemuck::cast_slice(&readback.slice(..).get_mapped_range().unwrap()).to_vec();

  words
}

#[test]
fn mirrors_a_store_through_adds_changes_removes_and_growth() {
  let Some((device, queue)) = create_device() else {
    return;
  };
  let mut store: ProxyStore<u32> = ProxyStore::new();
  let mut mirror: StoreMirror = StoreMirror::new(&device, "test", 4, wgpu::BufferUsages::STORAGE);
  // Past the mirror's first room, so it grows.
  let handles: Vec<ProxyHandle<u32>> = (0..100).map(|value| store.add(value)).collect();

  mirror.sync(&device, &queue, &mut store, |value| *value * 10);

  assert_eq!(mirror.get_generation(), 1);
  assert_eq!(mirror.get_count(), 100);
  assert_eq!(read_back(&device, &queue, &mirror, 100)[99], 990);

  *store.get_mut(handles[5]).unwrap() = 7;
  store.remove(handles[0]);
  mirror.sync(&device, &queue, &mut store, |value| *value * 10);

  let records: Vec<u32> = read_back(&device, &queue, &mirror, 99);

  assert_eq!(mirror.get_count(), 99);
  assert_eq!(records[0], 990, "the last record moved into the removed one's place");
  assert_eq!(records[5], 70);
  assert_eq!(records[98], 980);
}
