use std::path::{Path, PathBuf};

use xrf_dlss::Dlss;

use crate::context::gpu_context::GpuContext;
use crate::contract::render_libraries::RenderLibraries;
use crate::host::render_bundle::RenderBundle;

/// Describes this build's graphics libraries, looking for the DLSS runtime in `bundle`.
pub(crate) fn describe_libraries(bundle: &dyn RenderBundle) -> RenderLibraries {
  let library: Option<String> = Dlss::IS_BUILT
    .then(|| locate_dlss_runtime(bundle))
    .flatten()
    .and_then(|path| read_file_version(&path));

  RenderLibraries { dlss: library }
}

/// The DLSS runtime library `bundle` ships, none where it ships none.
pub(crate) fn locate_dlss_runtime(bundle: &dyn RenderBundle) -> Option<PathBuf> {
  bundle.locate_bundled(&format!("{}/{}", GpuContext::DLSS_DIRECTORY, Dlss::RUNTIME_LIBRARY))
}

/// Reads a Windows library's file version (`major.minor.build.revision`) from its version resource; none if absent.
#[cfg(windows)]
pub(crate) fn read_file_version(path: &Path) -> Option<String> {
  use std::os::windows::ffi::OsStrExt;

  use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VS_FIXEDFILEINFO, VerQueryValueW,
  };
  use windows::core::{PCWSTR, w};

  let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
  let name: PCWSTR = PCWSTR(wide.as_ptr());
  // SAFETY: `name` is a NUL-terminated path that outlives each call, and `block` is the size the first call asks. The
  // fixed info `VerQueryValueW` points at lies inside `block`, which outlives the read; it is copied out unaligned, as a
  // `Vec<u8>` promises no alignment, and only once the length it answers covers it.
  unsafe {
    let size: u32 = GetFileVersionInfoSizeW(name, None);

    if size == 0 {
      return None;
    }

    let mut block: Vec<u8> = vec![0; size as usize];

    GetFileVersionInfoW(name, None, size, block.as_mut_ptr().cast()).ok()?;

    let mut info: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut length: u32 = 0;

    if !VerQueryValueW(block.as_ptr().cast(), w!("\\"), &raw mut info, &raw mut length).as_bool()
      || info.is_null()
      || (length as usize) < size_of::<VS_FIXEDFILEINFO>()
    {
      return None;
    }

    let fixed: VS_FIXEDFILEINFO = std::ptr::read_unaligned(info as *const VS_FIXEDFILEINFO);

    Some(format!(
      "{}.{}.{}.{}",
      fixed.dwFileVersionMS >> 16,
      fixed.dwFileVersionMS & 0xffff,
      fixed.dwFileVersionLS >> 16,
      fixed.dwFileVersionLS & 0xffff
    ))
  }
}

/// Libraries have no version resource to read outside Windows.
#[cfg(not(windows))]
pub(crate) fn read_file_version(_path: &Path) -> Option<String> {
  None
}
