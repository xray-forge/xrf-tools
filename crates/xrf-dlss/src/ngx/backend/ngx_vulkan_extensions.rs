use std::ffi::CStr;
use std::sync::MutexGuard;
use std::{ptr, slice};

use wgpu::hal::vulkan::{CreateDeviceCallbackArgs, CreateInstanceCallbackArgs};
use xrf_error::{XrfError, XrfResult};

use crate::ngx::bindings::*;
use crate::ngx::ngx_application::NgxApplication;
use crate::ngx::ngx_result::NgxResult;
use crate::ngx::sdk::ngx_lock::NgxLock;

/// The Vulkan extensions NGX needs, added to an instance or device wgpu is creating, as named operations.
pub(crate) struct NgxVulkanExtensions;

impl NgxVulkanExtensions {
  /// Adds the instance extensions super resolution needs to an instance wgpu is creating, and ray reconstruction's
  /// where the loader has them all; whether it has super resolution's.
  pub(crate) fn add_to_instance(
    application: &NgxApplication,
    args: &mut CreateInstanceCallbackArgs<'_, '_, '_>,
  ) -> XrfResult<bool> {
    let supported = unsafe { args.entry.enumerate_instance_extension_properties(None) }
      .map_err(|error| XrfError::new_unexpected_error(format!("Vulkan lists no extensions: {error}")))?;
    let has = |names: &[&'static CStr]| {
      names.iter().all(|name| {
        supported
          .iter()
          .any(|extension| extension.extension_name_as_c_str() == Ok(*name))
      })
    };
    let required: Vec<&'static CStr> =
      Self::list_for_instance(application, NVSDK_NGX_Feature_NVSDK_NGX_Feature_SuperSampling)?;

    if !has(&required) {
      log::info!(
        "DLSS is unavailable: Vulkan's loader lacks {}",
        Self::list_missing(&required, |name| has(&[name]))
      );

      return Ok(false);
    }

    let reconstructing: Vec<&'static CStr> =
      Self::list_for_instance(application, NVSDK_NGX_Feature_NVSDK_NGX_Feature_RayReconstruction).unwrap_or_default();

    Self::add_unique(args.extensions, required);

    if has(&reconstructing) {
      Self::add_unique(args.extensions, reconstructing);
    }

    Ok(true)
  }

  /// Adds the device extensions super resolution needs to a device wgpu is opening, and ray reconstruction's where the
  /// adapter has them all; whether it has super resolution's.
  pub(crate) fn add_to_device(
    application: &NgxApplication,
    args: &mut CreateDeviceCallbackArgs<'_, '_, '_>,
    adapter: &wgpu::hal::vulkan::Adapter,
  ) -> XrfResult<bool> {
    let has = |names: &[&'static CStr]| {
      names
        .iter()
        .all(|name| adapter.physical_device_capabilities().supports_extension(name))
    };
    let required: Vec<&'static CStr> =
      Self::list_for_device(application, adapter, NVSDK_NGX_Feature_NVSDK_NGX_Feature_SuperSampling)?;

    if !has(&required) {
      log::info!(
        "DLSS is unavailable: the Vulkan adapter lacks {}",
        Self::list_missing(&required, |name| has(&[name]))
      );

      return Ok(false);
    }

    let reconstructing: Vec<&'static CStr> = Self::list_for_device(
      application,
      adapter,
      NVSDK_NGX_Feature_NVSDK_NGX_Feature_RayReconstruction,
    )
    .unwrap_or_default();

    Self::add_unique(args.extensions, required);

    if has(&reconstructing) {
      Self::add_unique(args.extensions, reconstructing);
    }

    Ok(true)
  }

  /// The instance extensions NGX names for a feature.
  fn list_for_instance(application: &NgxApplication, feature: NVSDK_NGX_Feature) -> XrfResult<Vec<&'static CStr>> {
    application.with_feature_info(feature, |info| unsafe {
      let _ngx: MutexGuard<'static, ()> = NgxLock::hold();
      let mut extensions: *mut ash::vk::ExtensionProperties = ptr::null_mut();
      let mut count: u32 = 0;

      NgxResult::check(
        NVSDK_NGX_VULKAN_GetFeatureInstanceExtensionRequirements(info, &mut count, &mut extensions),
        "NGX names no instance extensions",
      )?;

      Ok(Self::to_names(extensions, count))
    })
  }

  /// The device extensions NGX names for a feature on the adapter.
  fn list_for_device(
    application: &NgxApplication,
    adapter: &wgpu::hal::vulkan::Adapter,
    feature: NVSDK_NGX_Feature,
  ) -> XrfResult<Vec<&'static CStr>> {
    let instance: ash::vk::Instance = adapter.shared_instance().raw_instance().handle();
    let physical_device: ash::vk::PhysicalDevice = adapter.raw_physical_device();

    application.with_feature_info(feature, |info| unsafe {
      let _ngx: MutexGuard<'static, ()> = NgxLock::hold();
      let mut extensions: *mut ash::vk::ExtensionProperties = ptr::null_mut();
      let mut count: u32 = 0;

      NgxResult::check(
        NVSDK_NGX_VULKAN_GetFeatureDeviceExtensionRequirements(
          instance,
          physical_device,
          info,
          &mut count,
          &mut extensions,
        ),
        "NGX names no device extensions",
      )?;

      Ok(Self::to_names(extensions, count))
    })
  }

  /// Adds the names not already in `extensions`.
  fn add_unique(extensions: &mut Vec<&'static CStr>, names: Vec<&'static CStr>) {
    for name in names {
      if !extensions.contains(&name) {
        extensions.push(name);
      }
    }
  }

  /// The names of the extensions NGX lists, which it keeps for the process's life.
  ///
  /// # Safety
  ///
  /// `extensions` is `count` extensions NGX returned.
  unsafe fn to_names(extensions: *mut ash::vk::ExtensionProperties, count: u32) -> Vec<&'static CStr> {
    if count == 0 {
      return Vec::new();
    }

    unsafe { slice::from_raw_parts(extensions, count as usize) }
      .iter()
      .map(|extension| unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) })
      .collect()
  }

  /// The extensions of `names` that `is_there` finds missing, named and joined for a log line.
  fn list_missing(names: &[&'static CStr], is_there: impl Fn(&'static CStr) -> bool) -> String {
    names
      .iter()
      .filter(|name| !is_there(name))
      .map(|name| name.to_string_lossy())
      .collect::<Vec<_>>()
      .join(", ")
  }
}
