use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};
use xrf_error::{XrfError, XrfResult};

use crate::device::audio_device_description::AudioDeviceDescription;

/// The system's output devices.
pub struct AudioDevices;

impl AudioDevices {
  /// Every output device the system offers now, the default one marked.
  ///
  /// # Errors
  ///
  /// Returns an error when the system's audio host cannot list its devices.
  pub fn list() -> XrfResult<Vec<AudioDeviceDescription>> {
    let host: Host = cpal::default_host();
    let default_id: Option<String> = host.default_output_device().as_ref().and_then(Self::get_id);
    let devices = host
      .output_devices()
      .map_err(|error| XrfError::new_read_error(format!("Could not list output devices: {error}")))?;

    Ok(
      devices
        .filter_map(|device| {
          let id: String = Self::get_id(&device)?;

          Some(AudioDeviceDescription {
            name: Self::get_name(&device),
            is_default: default_id.as_ref() == Some(&id),
            id,
          })
        })
        .collect(),
    )
  }

  /// The device with `id`, if the system offers it now.
  pub(crate) fn find(host: &Host, id: &str) -> Option<Device> {
    host
      .output_devices()
      .ok()?
      .find(|device| Self::get_id(device).as_deref() == Some(id))
  }

  pub(crate) fn get_id(device: &Device) -> Option<String> {
    device.id().ok().map(|id| id.to_string())
  }

  pub(crate) fn get_name(device: &Device) -> String {
    device.description().map_or_else(
      |_| String::from("Unnamed device"),
      |description| description.name().to_owned(),
    )
  }
}
