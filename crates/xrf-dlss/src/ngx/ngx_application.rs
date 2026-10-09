use std::ffi::{CStr, CString, OsStr, c_char};
use std::path::{Path, PathBuf};
use std::ptr;

use crate::ngx::bindings::*;

/// The application as NGX knows it: its project, its engine, and the directories it loads the features' runtime
/// libraries from.
#[derive(Clone, Debug)]
pub(crate) struct NgxApplication {
  runtime: Vec<PathBuf>,
}

impl NgxApplication {
  /// The application's NGX project identifier.
  const PROJECT: &'static CStr = c"7a00e63c-9ae5-469d-b829-c449b105a48a";

  /// The application loading the runtime library from `runtime`, where it ships one, else from the SDK it was built
  /// against.
  pub(crate) fn new(runtime: Option<&Path>) -> Self {
    Self {
      runtime: runtime
        .map(to_plain)
        .into_iter()
        .chain([PathBuf::from(env!("XRF_DLSS_RUNTIME"))])
        .collect(),
    }
  }

  pub(crate) fn get_project(&self) -> &'static CStr {
    Self::PROJECT
  }

  pub(crate) fn get_engine_version(&self) -> CString {
    CString::new(env!("CARGO_PKG_VERSION")).expect("the package's version holds no zero byte")
  }

  /// Runs `callback` with NGX's description of the application and `feature`, which lives only for the call.
  pub(crate) fn with_feature_info<T>(
    &self,
    feature: NVSDK_NGX_Feature,
    callback: impl FnOnce(&NVSDK_NGX_FeatureDiscoveryInfo) -> T,
  ) -> T {
    let engine_version: CString = self.get_engine_version();
    let data_path: Vec<wchar_t> = to_wide(std::env::temp_dir().as_os_str());
    let runtime: Vec<Vec<wchar_t>> = self.runtime.iter().map(|path| to_wide(path.as_os_str())).collect();
    let runtime_pointers: Vec<*const wchar_t> = runtime.iter().map(|path| path.as_ptr()).collect();
    let common = NVSDK_NGX_FeatureCommonInfo {
      PathListInfo: NVSDK_NGX_PathListInfo {
        Path: runtime_pointers.as_ptr(),
        Length: runtime_pointers.len() as u32,
      },
      InternalData: ptr::null_mut(),
      // NGX's own messages go to the log while its debug level is on.
      LoggingInfo: NVSDK_NGX_LoggingInfo {
        LoggingCallback: Some(log_ngx),
        MinimumLoggingLevel: if log::log_enabled!(target: NGX_LOG, log::Level::Debug) {
          NVSDK_NGX_Logging_Level_NVSDK_NGX_LOGGING_LEVEL_VERBOSE
        } else {
          NVSDK_NGX_Logging_Level_NVSDK_NGX_LOGGING_LEVEL_OFF
        },
        DisableOtherLoggingSinks: true,
      },
    };
    let info = NVSDK_NGX_FeatureDiscoveryInfo {
      SDKVersion: NVSDK_NGX_Version_NVSDK_NGX_Version_API,
      FeatureID: feature,
      Identifier: NVSDK_NGX_Application_Identifier {
        IdentifierType: NVSDK_NGX_Application_Identifier_Type_NVSDK_NGX_Application_Identifier_Type_Project_Id,
        v: NVSDK_NGX_Application_Identifier_v {
          ProjectDesc: NVSDK_NGX_ProjectIdDescription {
            ProjectId: Self::PROJECT.as_ptr(),
            EngineType: NVSDK_NGX_EngineType_NVSDK_NGX_ENGINE_TYPE_CUSTOM,
            EngineVersion: engine_version.as_ptr(),
          },
        },
      },
      ApplicationDataPath: data_path.as_ptr(),
      FeatureInfo: &common,
    };

    callback(&info)
  }
}

/// A path as NGX reads it: a local one without Windows' `\\?\` verbatim prefix, which NGX finds nothing under.
fn to_plain(path: &Path) -> PathBuf {
  let text = path.to_string_lossy();

  match text.strip_prefix(r"\\?\") {
    Some(plain) if !plain.starts_with(r"UNC\") => PathBuf::from(plain),
    _ => path.to_path_buf(),
  }
}

/// The log target NGX's messages are written under.
const NGX_LOG: &str = "xrf_dlss::ngx";

/// Writes one of NGX's messages to the log.
unsafe extern "C" fn log_ngx(message: *const c_char, _level: NVSDK_NGX_Logging_Level, _source: NVSDK_NGX_Feature) {
  if message.is_null() {
    return;
  }

  let text = unsafe { CStr::from_ptr(message) }.to_string_lossy();

  log::debug!(target: NGX_LOG, "{}", text.trim_end());
}

#[cfg(target_os = "windows")]
fn to_wide(text: &OsStr) -> Vec<wchar_t> {
  use std::os::windows::ffi::OsStrExt;

  text.encode_wide().chain([0]).map(|unit| unit as wchar_t).collect()
}

#[cfg(not(target_os = "windows"))]
fn to_wide(text: &OsStr) -> Vec<wchar_t> {
  text
    .to_string_lossy()
    .chars()
    .chain(['\0'])
    .map(|unit| unit as wchar_t)
    .collect()
}
