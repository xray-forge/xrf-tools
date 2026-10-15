use std::ffi::CStr;
use std::sync::MutexGuard;
use std::{ptr, slice};

use wgpu::hal::vulkan::{CreateDeviceCallbackArgs, CreateInstanceCallbackArgs};
use xrf_error::XrfResult;

use crate::ngx::bindings::*;
use crate::ngx::ngx_application::NgxApplication;
use crate::ngx::ngx_lock::lock_ngx;
use crate::ngx::ngx_result::check_ngx;

/// Adds the instance extensions super resolution needs to an instance wgpu is creating; whether the loader has them.
pub(crate) fn add_instance_extensions(
  application: &NgxApplication,
  args: &mut CreateInstanceCallbackArgs<'_, '_, '_>,
) -> XrfResult<bool> {
  let required: Vec<&'static CStr> =
    application.with_feature_info(NVSDK_NGX_Feature_NVSDK_NGX_Feature_SuperSampling, |info| unsafe {
      let _ngx: MutexGuard<'static, ()> = lock_ngx();
      let mut extensions: *mut ash::vk::ExtensionProperties = ptr::null_mut();
      let mut count: u32 = 0;

      check_ngx(
        NVSDK_NGX_VULKAN_GetFeatureInstanceExtensionRequirements(info, &mut count, &mut extensions),
        "NGX names no instance extensions",
      )?;

      Ok::<_, xrf_error::XrfError>(to_names(extensions, count))
    })?;
  let supported = unsafe { args.entry.enumerate_instance_extension_properties(None) }
    .map_err(|error| xrf_error::XrfError::new_unexpected_error(format!("Vulkan lists no extensions: {error}")))?;
  let is_supported: bool = required.iter().all(|name| {
    supported
      .iter()
      .any(|extension| extension.extension_name_as_c_str() == Ok(*name))
  });

  if is_supported {
    args.extensions.extend(required);
  }

  Ok(is_supported)
}

/// Adds the device extensions super resolution needs to a device wgpu is opening; whether the adapter has them.
pub(crate) fn add_device_extensions(
  application: &NgxApplication,
  args: &mut CreateDeviceCallbackArgs<'_, '_, '_>,
  adapter: &wgpu::hal::vulkan::Adapter,
) -> XrfResult<bool> {
  let instance: ash::vk::Instance = adapter.shared_instance().raw_instance().handle();
  let physical_device: ash::vk::PhysicalDevice = adapter.raw_physical_device();
  let required: Vec<&'static CStr> =
    application.with_feature_info(NVSDK_NGX_Feature_NVSDK_NGX_Feature_SuperSampling, |info| unsafe {
      let _ngx: MutexGuard<'static, ()> = lock_ngx();
      let mut extensions: *mut ash::vk::ExtensionProperties = ptr::null_mut();
      let mut count: u32 = 0;

      check_ngx(
        NVSDK_NGX_VULKAN_GetFeatureDeviceExtensionRequirements(
          instance,
          physical_device,
          info,
          &mut count,
          &mut extensions,
        ),
        "NGX names no device extensions",
      )?;

      Ok::<_, xrf_error::XrfError>(to_names(extensions, count))
    })?;
  let is_supported: bool = required
    .iter()
    .all(|name| adapter.physical_device_capabilities().supports_extension(name));

  if is_supported {
    args.extensions.extend(required);
  }

  Ok(is_supported)
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
