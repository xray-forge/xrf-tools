use crate::pass::trace_request::TraceRequest;

/// The rays a pass leaves to the scene pass each frame: the requests, as many as `capacity` and one workgroup's at the
/// least; their head (the count made, the scene pass's dispatch and the wait, `TraceRequest::HEAD_WORDS`); and the
/// dispatch copied out of it.
pub struct TraceRequestList {
  label: String,
  capacity: u32,
  pub requests: wgpu::Buffer,
  pub head: wgpu::Buffer,
  pub dispatch: wgpu::Buffer,
}

impl TraceRequestList {
  /// Bytes its buffers take.
  pub fn get_memory(&self) -> u64 {
    self.requests.size() + self.head.size() + self.dispatch.size()
  }

  pub fn new(device: &wgpu::Device, label: &str, capacity: u32) -> Self {
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
    }
  }

  /// Requests it holds at most.
  pub fn get_capacity(&self) -> u32 {
    self.capacity
  }

  /// Makes the requests anew where they hold another number: the full share while the pass traces, none while it does
  /// not, so a list takes no memory until tracing is asked for.
  pub fn fit(&mut self, device: &wgpu::Device, capacity: u32) {
    if capacity != self.capacity {
      self.capacity = capacity;
      self.requests = Self::create_requests(device, &self.label, capacity);
    }
  }

  fn create_requests(device: &wgpu::Device, label: &str, capacity: u32) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some(&format!("{label} requests")),
      size: u64::from(capacity.max(TraceRequest::WORKGROUP)) * size_of::<TraceRequest>() as u64,
      usage: wgpu::BufferUsages::STORAGE,
      mapped_at_creation: false,
    })
  }
}
