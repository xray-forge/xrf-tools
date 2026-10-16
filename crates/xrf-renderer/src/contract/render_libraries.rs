use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_dlss::Dlss;

use crate::context::gpu_context::GpuContext;
use crate::host::render_bundle::RenderBundle;

/// Graphics libraries shipped beside the application; currently only the DLSS runtime.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLibraries {
  /// The DLSS runtime's file version, for example `310.9.1.0`; none if the build has no DLSS or the library is missing
  /// from the bundle.
  pub dlss: Option<String>,
}

impl RenderLibraries {
  /// Describes this build's libraries, looking for the DLSS runtime in `bundle`.
  pub fn describe(bundle: &dyn RenderBundle) -> Self {
    let library: Option<String> = Dlss::IS_BUILT
      .then(|| bundle.locate_bundled(&format!("{}/{}", GpuContext::DLSS_DIRECTORY, Dlss::RUNTIME_LIBRARY)))
      .flatten()
      .and_then(|path| Self::read_file_version(&path));

    Self { dlss: library }
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

  /// Libraries have no version resource to read outside Windows.
  #[cfg(not(windows))]
  pub(crate) fn read_file_version(_path: &Path) -> Option<String> {
    None
  }
}
