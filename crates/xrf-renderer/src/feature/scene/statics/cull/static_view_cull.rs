use crate::scene::static_scene::draw::static_cull_scratch::StaticCullScratch;

/// What a view's camera cull writes and its draws read: the visible list, the candidates its early phase sets aside for
/// the late, its draw and late arguments, and the late phase's dispatch. Each view holds its own, so the views of one
/// scene cull and draw apart whatever order a frame declares them in; the scene keeps what every cull starts from, the
/// lists' regions and the arguments' templates (`StaticCullScratch`).
pub struct StaticViewCull {
  pub lists: wgpu::Buffer,
  /// Clusters the early phase set aside, for the late phase to test again: as many entries as the visible list.
  pub candidates: wgpu::Buffer,
  pub args: wgpu::Buffer,
  /// Each batch's late draw arguments, then the late phase's dispatch and the candidates' count.
  pub late: wgpu::Buffer,
  /// The late phase's dispatch, copied out of `late` so the dispatch reads no buffer it binds.
  pub late_dispatch: wgpu::Buffer,
}

impl StaticViewCull {
  /// Buffers whose lists hold `list_bytes` (`StaticCullScratch::get_list_bytes`).
  pub fn new(device: &wgpu::Device, list_bytes: u64) -> Self {
    let drawn: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE
      | wgpu::BufferUsages::INDIRECT
      | wgpu::BufferUsages::COPY_DST
      | wgpu::BufferUsages::COPY_SRC;
    let buffer = |label: &str, size: u64, usage: wgpu::BufferUsages| {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
      })
    };
    let [lists, candidates] = ["static lists", "static candidates"]
      .map(|label| buffer(label, Self::get_list_size(list_bytes), wgpu::BufferUsages::STORAGE));

    Self {
      lists,
      candidates,
      args: buffer("static draw arguments", StaticCullScratch::ARGS_SIZE, drawn),
      late: buffer("static late draw arguments", StaticCullScratch::LATE_SIZE, drawn),
      late_dispatch: buffer(
        "static late dispatch",
        12,
        wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
      ),
    }
  }

  /// Makes `held` hold lists of at least `list_bytes`, made again doubled where they are too small; a cull fills them
  /// whole, so nothing they held is kept.
  pub fn fit(held: &mut Option<Self>, device: &wgpu::Device, list_bytes: u64) {
    match held {
      Some(cull) if cull.lists.size() >= list_bytes => {}
      _ => {
        let doubled: u64 = held.as_ref().map_or(0, |cull| cull.lists.size() * 2);

        *held = Some(Self::new(device, list_bytes.max(doubled)));
      }
    }
  }

  /// The bytes the lists are made at for `list_bytes`, aligned so a growing level remakes them seldom.
  fn get_list_size(list_bytes: u64) -> u64 {
    list_bytes.max(1).next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT * 64)
  }
}
