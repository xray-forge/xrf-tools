use wgpu::hal::api::Dx12;
use windows_core::Interface;

use crate::ngx::bindings::ID3D12Resource;

/// A texture view's texture as NGX reads or writes it on D3D12: its resource, which NGX makes its own views of.
///
/// # Safety
///
/// The texture is a D3D12 one, alive while NGX records with the resource.
pub(crate) unsafe fn to_d3d12_resource(view: &wgpu::TextureView) -> *mut ID3D12Resource {
  unsafe {
    view
      .texture()
      .as_hal::<Dx12>()
      .expect("DLSS reads D3D12 textures")
      .raw_resource()
      .as_raw()
      .cast()
  }
}
