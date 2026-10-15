use std::collections::HashMap;

use crate::context::gpu_memory_budget::GpuMemoryBudget;
use crate::contract::render_memory_section::RenderMemorySection;

/// What the GPU allocator holds, asked of it now and then: allocated and reserved, the most it held allocated since
/// the renderer started, and what it holds allocated by the names the resources were made under; with what the GPU's
/// own memory holds and allows, where its driver says.
#[derive(Clone, Debug, Default)]
pub struct GpuMemory {
  pub allocated: u64,
  pub reserved: u64,
  pub peak: u64,
  pub sections: Vec<RenderMemorySection>,
  pub budget: Option<GpuMemoryBudget>,
}

impl GpuMemory {
  /// Sections listed, the largest first; the rest are summed into one.
  pub const SECTIONS_MOST: usize = 24;
  /// The section the rest are summed into.
  pub const OTHER: &'static str = "Other";

  /// What the allocator holds now, its peak carried on from `last`, and the adapter's memory; none where the backend
  /// reports nothing.
  pub fn read((adapter, device): (&wgpu::Adapter, &wgpu::Device), last: Option<&Self>) -> Option<Self> {
    let report: wgpu::AllocatorReport = device.generate_allocator_report()?;
    let allocated: u64 = report.total_allocated_bytes;

    Some(Self {
      allocated,
      reserved: report.total_reserved_bytes,
      peak: last.map_or(allocated, |last| last.peak.max(allocated)),
      sections: Self::list_sections(
        report
          .allocations
          .iter()
          .map(|allocation| (allocation.name.as_str(), allocation.size)),
      ),
      budget: GpuMemoryBudget::read(adapter),
    })
  }

  /// The allocations summed by their name, a trailing number and the space before it dropped so that the sides of a
  /// history and the faces of a map read as one: the `SECTIONS_MOST` largest, the rest in `OTHER`.
  pub fn list_sections<'a>(allocations: impl Iterator<Item = (&'a str, u64)>) -> Vec<RenderMemorySection> {
    let mut by_name: HashMap<String, (u64, u32)> = HashMap::new();

    for (name, size) in allocations {
      let entry: &mut (u64, u32) = by_name
        .entry(
          name
            .trim_end_matches(|it: char| it.is_ascii_digit())
            .trim_end()
            .to_string(),
        )
        .or_default();

      entry.0 += size;
      entry.1 += 1;
    }

    let mut sections: Vec<RenderMemorySection> = by_name
      .into_iter()
      .map(|(name, (bytes, count))| RenderMemorySection { name, bytes, count })
      .collect();

    sections.sort_by(|first, second| {
      second
        .bytes
        .cmp(&first.bytes)
        .then_with(|| first.name.cmp(&second.name))
    });

    if sections.len() > Self::SECTIONS_MOST {
      let rest: Vec<RenderMemorySection> = sections.split_off(Self::SECTIONS_MOST - 1);

      sections.push(RenderMemorySection {
        name: Self::OTHER.to_string(),
        bytes: rest.iter().map(|it| it.bytes).sum(),
        count: rest.iter().map(|it| it.count).sum(),
      });
    }

    sections
  }
}
