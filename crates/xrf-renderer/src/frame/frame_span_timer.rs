use crate::frame::gpu_readback::GpuReadback;

/// How long the GPU spends on each frame from its first command to its last, whether or not its passes are timed: a
/// stamp at the start of the frame's own encoder and one in a buffer submitted after every other, read back without
/// waiting a few frames late. A device without timestamps inside encoders measures nothing.
pub struct FrameSpanTimer {
  queries: Option<wgpu::QuerySet>,
  /// Each slot's two stamps resolved at its own aligned offset.
  resolve: wgpu::Buffer,
  slots: Vec<GpuReadback>,
  /// The slot the frame being recorded stamps into, where one was free.
  slot: Option<usize>,
  /// Milliseconds a timestamp tick lasts.
  period: f32,
}

impl FrameSpanTimer {
  /// Frames on their way back at once.
  const SLOTS: usize = 3;
  /// Bytes between two slots' resolved stamps: a query resolve's offset alignment.
  const STRIDE: u64 = wgpu::QUERY_RESOLVE_BUFFER_ALIGNMENT;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let queries: Option<wgpu::QuerySet> = device
      .features()
      .contains(wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
      .then(|| {
        device.create_query_set(&wgpu::QuerySetDescriptor {
          label: Some("frame span"),
          ty: wgpu::QueryType::Timestamp,
          count: Self::SLOTS as u32 * 2,
        })
      });

    Self {
      queries,
      resolve: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("frame span resolve"),
        size: Self::STRIDE * Self::SLOTS as u64,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
      }),
      slots: (0..Self::SLOTS)
        .map(|_| GpuReadback::new(device, "frame span readback", 16))
        .collect(),
      slot: None,
      period: queue.get_timestamp_period() / 1_000_000.0,
    }
  }

  /// Stamps the start of a frame into the first free slot, at the start of the frame's own encoder, which is submitted
  /// first; nothing while every slot is on its way back.
  pub fn begin(&mut self, encoder: &mut wgpu::CommandEncoder) {
    self.slot = None;

    let Some(queries) = &self.queries else {
      return;
    };
    let Some(slot) = self.slots.iter().position(GpuReadback::is_free) else {
      return;
    };

    encoder.write_timestamp(queries, slot as u32 * 2);
    self.slot = Some(slot);
  }

  /// The commands stamping the end of the frame and copying its two stamps out, to submit after all of the frame's
  /// others; none where its start was not stamped.
  pub fn end(&mut self, device: &wgpu::Device) -> Option<wgpu::CommandBuffer> {
    let (queries, slot) = (self.queries.as_ref()?, self.slot.take()?);
    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("frame span"),
    });
    let first: u32 = slot as u32 * 2;
    let offset: u64 = Self::STRIDE * slot as u64;

    encoder.write_timestamp(queries, first + 1);
    encoder.resolve_query_set(queries, first..first + 2, &self.resolve, offset);
    encoder.copy_buffer_to_buffer(&self.resolve, offset, self.slots[slot].get_buffer(), 0, 16);
    self.slots[slot].mark_recorded();

    Some(encoder.finish())
  }

  /// Asks for the stamps of the frame just submitted; call after each submit.
  pub fn request(&self) {
    for slot in &self.slots {
      slot.request();
    }
  }

  /// The milliseconds of the latest frame read back since the last call, none where none came back.
  pub fn take(&mut self) -> Option<f32> {
    let period: f32 = self.period;
    let mut latest: Option<f32> = None;

    for slot in &self.slots {
      if let Some(Ok(span)) = slot.take(|bytes| {
        let (stamps, _) = bytes.as_chunks::<8>();
        let (start, end) = (u64::from_le_bytes(stamps[0]), u64::from_le_bytes(stamps[1]));

        // A tick count running backwards is a driver's, not a negative span.
        end.saturating_sub(start) as f32 * period
      }) {
        latest = Some(span);
      }
    }

    latest
  }
}
