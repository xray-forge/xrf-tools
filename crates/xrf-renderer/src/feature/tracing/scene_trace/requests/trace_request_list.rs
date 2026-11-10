use crate::feature::tracing::scene_trace::requests::trace_request::TraceRequest;

/// The rays a pass leaves to the scene pass each frame: the requests, as many as `capacity` and one workgroup's at the
/// least; their head (the count made, the scene pass's dispatch and the wait, `TraceRequest::HEAD_WORDS`); the
/// dispatch copied out of it; and the wait copied out of it as the frame starts, which the listing pass reads plainly
/// rather than by an atomic load in every pixel.
pub(crate) struct TraceRequestList {
  label: String,
  capacity: u32,
  pub requests: wgpu::Buffer,
  pub head: wgpu::Buffer,
  pub dispatch: wgpu::Buffer,
  pub wait: wgpu::Buffer,
}

impl TraceRequestList {
  pub(crate) fn new(device: &wgpu::Device, label: &str, capacity: u32) -> Self {
    let buffer = |suffix: &str, size: u64, usage: wgpu::BufferUsages| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("{label} {suffix}")),
        size,
        usage,
        mapped_at_creation: false,
      })
    };

    Self {
      label: label.to_string(),
      capacity,
      requests: Self::create_requests(device, label, capacity),
      head: buffer(
        "request head",
        (TraceRequest::HEAD_WORDS * size_of::<u32>()) as u64,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
      ),
      dispatch: buffer(
        "request dispatch",
        TraceRequest::DISPATCH_BYTES,
        wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
      ),
      wait: buffer(
        "request wait",
        size_of::<u32>() as u64,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      ),
    }
  }

  /// Requests it holds at most.
  pub(crate) fn get_capacity(&self) -> u32 {
    self.capacity
  }

  /// Makes the requests anew where they hold another number: the full share while the pass traces, none while it does
  /// not, so a list takes no memory until tracing is asked for.
  pub(crate) fn fit(&mut self, device: &wgpu::Device, capacity: u32) {
    if capacity != self.capacity {
      self.capacity = capacity;
      self.requests = Self::create_requests(device, &self.label, capacity);
    }
  }

  /// The requests' buffer, as many as `capacity` and one workgroup's at the least, but no more than one binding of the
  /// device holds: a ray past its room is held as one past a full list is.
  fn create_requests(device: &wgpu::Device, label: &str, capacity: u32) -> wgpu::Buffer {
    let stride: u64 = size_of::<TraceRequest>() as u64;
    let most: u64 = device.limits().max_storage_buffer_binding_size / stride * stride;

    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some(&format!("{label} requests")),
      size: (u64::from(capacity.max(TraceRequest::WORKGROUP)) * stride).min(most),
      usage: wgpu::BufferUsages::STORAGE,
      mapped_at_creation: false,
    })
  }
}
