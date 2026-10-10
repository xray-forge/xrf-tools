use crate::contract::render_memory_section::RenderMemorySection;
use crate::frame::gpu_memory::GpuMemory;

// Allocations sum by their name, the sides of a history as one, the largest first.
#[test]
fn sums_allocations_by_their_name_the_largest_first() {
  let sections: Vec<RenderMemorySection> = GpuMemory::list_sections(
    [
      ("reflection history 0", 40),
      ("reflection history 1", 40),
      ("level texture", 30),
      ("level texture", 30),
      ("graph transient", 100),
    ]
    .into_iter(),
  );

  assert_eq!(
    sections,
    vec![
      RenderMemorySection {
        name: "graph transient".to_string(),
        bytes: 100,
        count: 1
      },
      RenderMemorySection {
        name: "reflection history".to_string(),
        bytes: 80,
        count: 2
      },
      RenderMemorySection {
        name: "level texture".to_string(),
        bytes: 60,
        count: 2
      },
    ]
  );
}

// Past the most listed, the smallest are summed into one.
#[test]
fn sums_the_smallest_into_one_past_the_most_listed() {
  let names: Vec<String> = (0..30)
    .map(|index| format!("section {}", char::from(b'a' + index as u8)))
    .collect();
  let sections: Vec<RenderMemorySection> = GpuMemory::list_sections(
    names
      .iter()
      .enumerate()
      .map(|(index, name)| (name.as_str(), 100 - index as u64)),
  );
  let other: &RenderMemorySection = sections.last().expect("a section");

  assert_eq!(sections.len(), GpuMemory::SECTIONS_MOST);
  assert_eq!(other.name, GpuMemory::OTHER);
  assert_eq!(other.count, 30 - (GpuMemory::SECTIONS_MOST as u32 - 1));
  assert_eq!(sections.iter().map(|it| it.bytes).sum::<u64>(), (71..=100).sum::<u64>());
}
