use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_dlss::Dlss;

use crate::context::gpu_context::GpuContext;
use crate::host::render_bundle::RenderBundle;

/// The graphics libraries the renderer ships beside the application: the DLSS runtime.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLibraries {
  /// The DLSS runtime's own file version, such as `310.9.1.0`; none where the build has no DLSS or its library is not
  /// in the bundle.
  pub dlss: Option<String>,
}

impl RenderLibraries {
  /// The libraries as this build has them, the DLSS runtime looked for in `bundle`.
  pub fn describe(bundle: &dyn RenderBundle) -> Self {
    let library: Option<String> = Dlss::IS_BUILT
      .then(|| bundle.locate_bundled(&format!("{}/{}", GpuContext::DLSS_DIRECTORY, Dlss::RUNTIME_LIBRARY)))
      .flatten()
      .and_then(|path| Self::read_file_version(&path));

    Self { dlss: library }
  }

  /// A Windows library's file version from its version resource, `major.minor.build.revision`; none where it has none.
  #[cfg(windows)]
  pub(crate) fn read_file_version(path: &Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;

    use windows::Win32::Storage::FileSystem::{
      GetFileVersionInfoSizeW, GetFileVersionInfoW, VS_FIXEDFILEINFO, VerQueryValueW,
    };
    use windows::core::{PCWSTR, w};

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let name: PCWSTR = PCWSTR(wide.as_ptr());
    // SAFETY: `name` is a NUL-terminated path that outlives each call, and `block` is the size the first call asks.
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

      let fixed: &VS_FIXEDFILEINFO = &*(info as *const VS_FIXEDFILEINFO);

      Some(format!(
        "{}.{}.{}.{}",
        fixed.dwFileVersionMS >> 16,
        fixed.dwFileVersionMS & 0xffff,
        fixed.dwFileVersionLS >> 16,
        fixed.dwFileVersionLS & 0xffff
      ))
    }
  }

  /// No version resource to read off Windows.
  #[cfg(not(windows))]
  pub(crate) fn read_file_version(_path: &Path) -> Option<String> {
    None
  }
}
