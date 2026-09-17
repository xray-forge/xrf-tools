use xrf_extension::XrayExtension;

/// Extensions a viewer reads as text rather than as binary.
pub const ALLOWED_TEXT_EXTENSIONS: &[XrayExtension] = &[
  XrayExtension::Bat,
  XrayExtension::Cmd,
  XrayExtension::Cs,
  XrayExtension::Ds,
  XrayExtension::Gs,
  XrayExtension::H,
  XrayExtension::Hlsl,
  XrayExtension::Hs,
  XrayExtension::Ini,
  XrayExtension::Json,
  XrayExtension::Log,
  XrayExtension::Ltx,
  XrayExtension::Lua,
  XrayExtension::Md,
  XrayExtension::Ps,
  XrayExtension::Py,
  XrayExtension::S,
  XrayExtension::Script,
  XrayExtension::Seq,
  XrayExtension::Vs,
  XrayExtension::Xml,
];

pub const ALLOWED_TEXT_SIZE: u32 = 10 * 1024 * 1024; // 10 MBytes

/// Extensions the backend decodes into a picture, which is the one the engine loads.
pub const ALLOWED_TEXTURE_EXTENSIONS: &[XrayExtension] = &[XrayExtension::Dds];

/// Extensions the webview renders as they stand, so the backend only has to hand the bytes over.
pub const ALLOWED_IMAGE_EXTENSIONS: &[XrayExtension] = &[
  XrayExtension::Bmp,
  XrayExtension::Jpeg,
  XrayExtension::Jpg,
  XrayExtension::Png,
];

/// Upper bound on a picture handed over whole, which is the same bound a decoded one answers to.
pub const ALLOWED_IMAGE_SIZE: u32 = 32 * 1024 * 1024; // 32 MBytes

/// Upper bound on a texture entry, guarding against holding a very large one in memory to decode.
pub const ALLOWED_TEXTURE_SIZE: u32 = 32 * 1024 * 1024; // 32 MBytes

/// Extensions the backend hands to the webview to play rather than decoding itself.
pub const ALLOWED_AUDIO_EXTENSIONS: &[XrayExtension] = &[XrayExtension::Ogg, XrayExtension::Wav];

/// Upper bound on an audio entry, which is held whole in memory on the way to the webview.
pub const ALLOWED_AUDIO_SIZE: u32 = 64 * 1024 * 1024; // 64 MBytes

/// Upper bound on an entry read whole so its format can be described.
pub const ALLOWED_DESCRIBE_SIZE: u32 = 64 * 1024 * 1024; // 64 MBytes

/// Upper bound on an entry read whole only to walk the container it is.
pub const ALLOWED_CHUNK_TREE_SIZE: u32 = 8 * 1024 * 1024; // 8 MBytes
