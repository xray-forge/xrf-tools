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

/// Extensions the backend can decode into a picture rather than read as text.
pub const ALLOWED_IMAGE_EXTENSIONS: &[XrayExtension] = &[XrayExtension::Dds];

/// Upper bound on an image entry, guarding against holding a very large texture in memory to decode.
pub const ALLOWED_IMAGE_SIZE: u32 = 32 * 1024 * 1024; // 32 MBytes

/// Extensions the backend hands to the webview to play rather than decoding itself.
pub const ALLOWED_AUDIO_EXTENSIONS: &[XrayExtension] = &[XrayExtension::Ogg];

/// Upper bound on an audio entry, which is held whole in memory on the way to the webview.
pub const ALLOWED_AUDIO_SIZE: u32 = 64 * 1024 * 1024; // 64 MBytes

/// Upper bound on an entry read whole so its format can be described.
pub const ALLOWED_DESCRIBE_SIZE: u32 = 64 * 1024 * 1024; // 64 MBytes

/// Upper bound on an entry read whole only to walk the container it is.
pub const ALLOWED_CHUNK_TREE_SIZE: u32 = 8 * 1024 * 1024; // 8 MBytes
