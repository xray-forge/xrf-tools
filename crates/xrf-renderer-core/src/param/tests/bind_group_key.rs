//! What a cached bind group is found by: a hash and a match read from a frame's bindings as they are.

use crate::param::bind_group_key::BindGroupKey;
use crate::param::pass_binding::PassBinding;

/// The same resources bound again, a view array among them, hash alike and match the key made for them; another view in
/// the array, or the same buffer bound as a range, does not match.
#[test]
fn finds_the_key_of_the_same_bindings_and_no_other() {
  let Some((device, _)) = crate::tests::test_device::create_device() else {
    return;
  };
  let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("key"),
    entries: &[],
  });
  let buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("key"),
    size: 256,
    usage: wgpu::BufferUsages::STORAGE,
    mapped_at_creation: false,
  });
  let view = || {
    device
      .create_texture(&wgpu::TextureDescriptor {
        label: Some("key"),
        size: wgpu::Extent3d {
          width: 4,
          height: 4,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
      .create_view(&Default::default())
  };
  let views: [wgpu::TextureView; 3] = [view(), view(), view()];
  let bound: Vec<PassBinding<'_>> = vec![
    PassBinding::Buffer(&buffer),
    PassBinding::TextureViewArray(vec![&views[0], &views[1]]),
  ];
  let again: Vec<PassBinding<'_>> = vec![
    PassBinding::Buffer(&buffer),
    PassBinding::TextureViewArray(vec![&views[0], &views[1]]),
  ];
  let other_view: Vec<PassBinding<'_>> = vec![
    PassBinding::Buffer(&buffer),
    PassBinding::TextureViewArray(vec![&views[0], &views[2]]),
  ];
  let ranged: Vec<PassBinding<'_>> = vec![
    PassBinding::BufferRange {
      buffer: &buffer,
      offset: 0,
      size: 256,
    },
    PassBinding::TextureViewArray(vec![&views[0], &views[1]]),
  ];
  let key: BindGroupKey = BindGroupKey::new(&layout, &bound);

  assert_eq!(
    BindGroupKey::hash_of(&layout, &again),
    BindGroupKey::hash_of(&layout, &bound)
  );
  assert!(key.is_of(&layout, &again));
  assert!(!key.is_of(&layout, &other_view));
  assert!(!key.is_of(&layout, &ranged));
}
