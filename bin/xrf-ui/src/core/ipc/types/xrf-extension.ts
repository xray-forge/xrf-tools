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
export type XrayExtension =
  | "ai"
  | "anm"
  | "anm1"
  | "bat"
  | "bmp"
  | "cform"
  | "cmd"
  | "cs"
  | "dds"
  | "details"
  | "dm"
  | "ds"
  | "efd"
  | "env_mod"
  | "fog_vol"
  | "game"
  | "geom"
  | "geomx"
  | "gs"
  | "h"
  | "hom"
  | "htm"
  | "html"
  | "hs"
  | "ini"
  | "json"
  | "lights"
  | "log"
  | "ltx"
  | "md"
  | "ogf"
  | "ogg"
  | "ogm"
  | "omf"
  | "png"
  | "ppe"
  | "ps"
  | "ps_static"
  | "py"
  | "s"
  /**
   * The trailing underscore is the spelling, not a typo the vocabulary should tidy: `s_` sits beside `s` in a
   * shaders tree and the engine's own readers treat it as another shader source.
   */
  | "s_"
  | "script"
  | "seq"
  /** Named the way `s_` is, beside `seq` in the same trees. */
  | "seq_"
  | "snd_static"
  | "spawn"
  | "tga"
  | "thm"
  /**
   * XRF's own source spelling for a generated config, which `xrf-ltx` looks for beside an absent `.ltx` to tell a
   * config that has not been built yet from one that is missing.
   */
  | "ts"
  | "vs"
  | "wallmarks"
  | "xml"
  /** The shader library container, `shaders.xr`. Nothing else in a tree carries it. */
  | "xr";
