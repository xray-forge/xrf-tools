use xrf_extension::XrayExtension;

/// Extensions a viewer reads as text rather than as binary.
///
/// Engine formats, the shader sources beside them, and the plain text a mod ships its notes and manifests in. `S` also
/// covers `shaders\r1\.s`, whose whole name is its extension.
pub const ALLOWED_TEXT_EXTENSIONS: &[XrayExtension] = &[
  XrayExtension::Cmd,
  XrayExtension::Ds,
  XrayExtension::H,
  XrayExtension::Hs,
  XrayExtension::Json,
  XrayExtension::Ltx,
  XrayExtension::Md,
  XrayExtension::Ps,
  XrayExtension::S,
  XrayExtension::Script,
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
