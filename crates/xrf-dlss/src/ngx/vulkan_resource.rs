use ash::vk::{ImageAspectFlags, ImageSubresourceRange, REMAINING_ARRAY_LAYERS, REMAINING_MIP_LEVELS};
use wgpu::hal::api::Vulkan;

use crate::ngx::bindings::*;

/// A texture view as NGX reads or writes it on Vulkan: the image, the view, its whole range, format and size, and
/// whether it is bound as storage.
///
/// # Safety
///
/// The view and its texture are Vulkan ones, alive while NGX records with the resource.
pub(crate) unsafe fn to_ngx_resource(view: &wgpu::TextureView, adapter: &wgpu::Adapter) -> NVSDK_NGX_Resource_VK {
  let texture: &wgpu::Texture = view.texture();
  // Each handle is taken in a statement of its own: a hal guard holds wgpu's lock until it drops, and two at once are
  // a recursive read lock.
  let image_view: ash::vk::ImageView =
    unsafe { view.as_hal::<Vulkan>().expect("DLSS reads Vulkan views").raw_handle() };
  let image: ash::vk::Image = unsafe {
    texture
      .as_hal::<Vulkan>()
      .expect("DLSS reads Vulkan textures")
      .raw_handle()
  };
  let format: ash::vk::Format = unsafe { adapter.as_hal::<Vulkan>() }
    .expect("DLSS runs on a Vulkan adapter")
    .texture_format_as_raw(texture.format());
  let range: ImageSubresourceRange = ImageSubresourceRange {
    aspect_mask: if texture.format().has_color_aspect() {
      ImageAspectFlags::COLOR
    } else {
      ImageAspectFlags::DEPTH
    },
    base_mip_level: 0,
    level_count: REMAINING_MIP_LEVELS,
    base_array_layer: 0,
    layer_count: REMAINING_ARRAY_LAYERS,
  };

  unsafe {
    NVSDK_NGX_Create_ImageView_Resource_VK(
      image_view,
      image,
      range,
      format,
      texture.width(),
      texture.height(),
      texture.usage().contains(wgpu::TextureUsages::STORAGE_BINDING),
    )
  }
}
