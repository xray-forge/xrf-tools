// Auto-generated rust bindings. Do not edit it manually.

/**
 * A file extension X-Ray game data or XRF tooling actually ships, as it is spelled on disk.
 *
 * One variant per real spelling, so `anm` and `anm1` are two members rather than one normalized to the other:
 * what a reader does with them is the reader's business, and two of them already disagree about it. Membership
 * is evidence — a tree that ships the spelling, or a reader that loads it — never a guess at what might exist.
 *
 * The serialized name is the spelling, written per variant rather than derived: a case convention would emit
 * `psStatic` for a file that is spelled `ps_static` on disk, and a consumer comparing against a name table would
 * then match nothing.
 */
export enum EXrayExtension {
  AI = "ai",
  ANM = "anm",
  ANM1 = "anm1",
  BAT = "bat",
  BMP = "bmp",
  C_FORM = "cform",
  CMD = "cmd",
  CS = "cs",
  DDS = "dds",
  DETAILS = "details",
  DM = "dm",
  DS = "ds",
  EFD = "efd",
  ENV_MOD = "env_mod",
  FOG_VOL = "fog_vol",
  GAME = "game",
  GEOM = "geom",
  GEOM_X = "geomx",
  GS = "gs",
  H = "h",
  /** The D3D11 shader sources IX-Ray ships loose in `shaders\d3d11\`, beside the older renderers' `.ps`/`.vs` pairs. */
  HLSL = "hlsl",
  HOM = "hom",
  HS = "hs",
  HTM = "htm",
  HTML = "html",
  INI = "ini",
  /** Both spellings of one format, because both reach a reader: `image::open` picks its decoder off the path. */
  JPEG = "jpeg",
  JPG = "jpg",
  JSON = "json",
  LIGHTS = "lights",
  LOG = "log",
  LTX = "ltx",
  LUA = "lua",
  MD = "md",
  OGF = "ogf",
  OGG = "ogg",
  OGM = "ogm",
  OMF = "omf",
  PNG = "png",
  PPE = "ppe",
  PS = "ps",
  PS_STATIC = "ps_static",
  PY = "py",
  S = "s",
  /**
   * The trailing underscore is the spelling, not a typo the vocabulary should tidy: `s_` sits beside `s` in a
   * shaders tree and the engine's own readers treat it as another shader source.
   */
  S_ = "s_",
  SCRIPT = "script",
  SEQ = "seq",
  /** Named the way `s_` is, beside `seq` in the same trees. */
  SEQ_ = "seq_",
  SND_STATIC = "snd_static",
  /** A level's sound occlusion mesh, which sits beside `hom` and was missing from this list. */
  SOM = "som",
  SPAWN = "spawn",
  TGA = "tga",
  THM = "thm",
  /**
   * XRF's own source spelling for a generated config, which `xrf-ltx` looks for beside an absent `.ltx` to tell a
   * config that has not been built yet from one that is missing.
   */
  TS = "ts",
  VS = "vs",
  WALLMARKS = "wallmarks",
  WAV = "wav",
  XML = "xml",
  /** The shader library container, `shaders.xr`. Nothing else in a tree carries it. */
  XR = "xr",
}

/** Every `EXrayExtension` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type XrayExtension = `${EXrayExtension}`;
