use xrf_error::{XrfError, XrfResult};

use crate::ngx::bindings::*;

/// What an NGX call answered, as named operations, since its result is a plain integer the bindings only alias.
pub(crate) struct NgxResult;

impl NgxResult {
  /// An NGX call's result: nothing where it succeeded, an error naming `what` was done and why NGX refused otherwise.
  pub(crate) fn check(result: NVSDK_NGX_Result, what: &str) -> XrfResult<()> {
    if result == NVSDK_NGX_Result_NVSDK_NGX_Result_Success {
      return Ok(());
    }

    Err(XrfError::new_unexpected_error(format!(
      "{what}: {} ({result:#x})",
      Self::describe(result)
    )))
  }

  /// NGX's reason for a failed result, as its headers explain it.
  #[allow(non_upper_case_globals)]
  fn describe(result: NVSDK_NGX_Result) -> &'static str {
    match result {
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_FeatureNotSupported => {
        "the feature is not supported by this system, GPU or graphics API"
      }
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_PlatformError => "the graphics API, driver or system failed",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_FeatureAlreadyExists => "an identical feature already exists",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_FeatureNotFound => "no feature has the handle",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_InvalidParameter => "a parameter is invalid or missing",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_ScratchBufferTooSmall => "the scratch buffer is too small",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_NotInitialized => "NGX is not initialized",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_UnsupportedInputFormat => "an input's format is unsupported",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_RWFlagMissing => "an output is not writable as storage",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_MissingInput => "an input is missing",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_UnableToInitializeFeature => {
        "the feature cannot start, its runtime library likely not found"
      }
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_OutOfDate => "the driver or the runtime library is too old",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_OutOfGPUMemory => "the GPU is out of memory",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_UnsupportedFormat => "a buffer's format is unsupported",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_UnableToWriteToAppDataPath => "the application data path is not writable",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_UnsupportedParameter => "a parameter is unsupported",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_Denied => "NVIDIA has denied the feature to this application",
      NVSDK_NGX_Result_NVSDK_NGX_Result_FAIL_NotImplemented => "the feature is not implemented in this version",
      _ => "an unknown NGX error",
    }
  }
}
