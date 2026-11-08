#![expect(unsafe_code, reason = "serves Steam Audio's C allocation callbacks")]

use std::alloc::Layout;
use std::ffi::c_void;

/// Steam Audio's memory, taken from the process's own allocator, so what it allocates is what Rust's allocator sees,
/// and an allocation on the audio thread is caught as any other is. Each block carries its layout ahead of it, since
/// Steam Audio frees by the pointer alone.
pub(crate) struct SteamAllocator;

impl SteamAllocator {
  /// Bytes kept ahead of each block, holding its layout.
  const HEADER: usize = 64;

  pub(crate) unsafe extern "C" fn allocate(size: usize, alignment: usize) -> *mut c_void {
    let alignment: usize = alignment.max(align_of::<usize>()).next_power_of_two();
    let header: usize = Self::HEADER.max(alignment);
    let Ok(layout) = Layout::from_size_align(size + header, alignment) else {
      return std::ptr::null_mut();
    };
    // SAFETY: the layout is not zero-sized: it holds the header at least.
    let base: *mut u8 = unsafe { std::alloc::alloc(layout) };

    if base.is_null() {
      return std::ptr::null_mut();
    }

    // SAFETY: the header is at least three words long and lies inside the block, before the pointer handed out, which
    // keeps its alignment since the header is a multiple of it.
    unsafe {
      let block: *mut u8 = base.add(header);
      let words: *mut usize = block.cast::<usize>();

      words.sub(1).write(layout.size());
      words.sub(2).write(alignment);
      words.sub(3).write(header);

      block.cast()
    }
  }

  /// Frees a block [`Self::allocate`] handed out.
  pub(crate) unsafe extern "C" fn free(block: *mut c_void) {
    if block.is_null() {
      return;
    }

    // SAFETY: the block came from `allocate`, which wrote its layout in the words before it.
    unsafe {
      let words: *mut usize = block.cast::<usize>();
      let size: usize = words.sub(1).read();
      let alignment: usize = words.sub(2).read();
      let header: usize = words.sub(3).read();

      std::alloc::dealloc(
        block.cast::<u8>().sub(header),
        Layout::from_size_align_unchecked(size, alignment),
      );
    }
  }
}
