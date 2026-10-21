/// The GPU's own memory as its driver tells it: all of it, what the system lets this process hold there now, and what
/// the process holds there, everything counted, DLSS's and the driver's with the renderer's.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GpuMemoryBudget {
  pub total: u64,
  pub budget: u64,
  pub used: u64,
}

impl GpuMemoryBudget {
  /// What the adapter's driver says of its own memory now: DXGI's on D3D12, `VK_EXT_memory_budget`'s over the
  /// device-local heaps on Vulkan, the device's working set on Metal; none where the driver says nothing.
  pub fn read(adapter: &wgpu::Adapter) -> Option<Self> {
    match adapter.get_info().backend {
      #[cfg(windows)]
      wgpu::Backend::Dx12 => Self::read_dx12(adapter),
      // wgpu builds Vulkan on Apple platforms only through MoltenVK's portability layer, which the workspace leaves off.
      #[cfg(not(target_vendor = "apple"))]
      wgpu::Backend::Vulkan => Self::read_vulkan(adapter),
      #[cfg(target_vendor = "apple")]
      wgpu::Backend::Metal => Self::read_metal(adapter),
      _ => None,
    }
  }

  #[cfg(windows)]
  fn read_dx12(adapter: &wgpu::Adapter) -> Option<Self> {
    use windows::Win32::Graphics::Dxgi::{DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_QUERY_VIDEO_MEMORY_INFO};

    // SAFETY: the raw adapter is only queried here, never retained or changed, so wgpu's own state is untouched.
    let hal = unsafe { adapter.as_hal::<wgpu::hal::api::Dx12>() }?;
    let raw = &**hal.raw_adapter();
    let mut info: DXGI_QUERY_VIDEO_MEMORY_INFO = DXGI_QUERY_VIDEO_MEMORY_INFO::default();

    // SAFETY: node 0 is every adapter's first, and `info` is a live, writable struct of the type DXGI fills.
    unsafe { raw.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &raw mut info) }.ok()?;

    // SAFETY: a read of the live adapter's description, which DXGI returns by value.
    let description = unsafe { raw.GetDesc1() }.ok()?;

    Some(Self {
      total: description.DedicatedVideoMemory as u64,
      budget: info.Budget,
      used: info.CurrentUsage,
    })
  }

  #[cfg(not(target_vendor = "apple"))]
  fn read_vulkan(adapter: &wgpu::Adapter) -> Option<Self> {
    use ash::vk;

    // SAFETY: the raw adapter is only queried here, never retained or changed, so wgpu's own state is untouched.
    let hal = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }?;

    if !hal
      .physical_device_capabilities()
      .supports_extension(ash::ext::memory_budget::NAME)
    {
      return None;
    }

    let instance: &ash::Instance = hal.shared_instance().raw_instance();
    let mut budgets: vk::PhysicalDeviceMemoryBudgetPropertiesEXT<'_> =
      vk::PhysicalDeviceMemoryBudgetPropertiesEXT::default();
    let mut properties: vk::PhysicalDeviceMemoryProperties2<'_> =
      vk::PhysicalDeviceMemoryProperties2::default().push_next(&mut budgets);

    // SAFETY: the physical device is the live instance's, and the budget struct chained into `properties` is only
    // written because the device supports `VK_EXT_memory_budget`, checked above.
    unsafe { instance.get_physical_device_memory_properties2(hal.raw_physical_device(), &mut properties) };

    let memory: vk::PhysicalDeviceMemoryProperties = properties.memory_properties;
    let heaps = memory.memory_heaps_as_slice();
    let mut total: Self = Self::default();

    for (index, heap) in heaps.iter().enumerate() {
      if heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
        total.total += heap.size;
        total.budget += budgets.heap_budget[index];
        total.used += budgets.heap_usage[index];
      }
    }

    Some(total)
  }

  /// Metal's memory is the machine's, shared with the CPU, so all of it the GPU may hold is the working set the device
  /// recommends staying within: it is both the total and the budget.
  #[cfg(target_vendor = "apple")]
  fn read_metal(adapter: &wgpu::Adapter) -> Option<Self> {
    use objc2_metal::MTLDevice;

    // SAFETY: the raw adapter is only queried here, never retained or changed, so wgpu's own state is untouched.
    let hal = unsafe { adapter.as_hal::<wgpu::hal::api::Metal>() }?;
    let device = hal.raw_device();
    let working_set: u64 = device.recommendedMaxWorkingSetSize();

    Some(Self {
      total: working_set,
      budget: working_set,
      used: device.currentAllocatedSize() as u64,
    })
  }
}
