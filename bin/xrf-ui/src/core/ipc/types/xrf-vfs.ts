// Auto-generated rust bindings. Do not edit it manually.

/** One asset a mount resolved: its engine identity plus the container it came out of. */
export type XrayAsset = {
  /** Lower-case, backslash-separated engine identity, including the mount's logical base. */
  logicalPath: XrayLogicalPath;
  /** Physical container reported by the source that resolved the asset. */
  container: XrayAssetContainer;
};

/** Every `kind` the `XrayAssetContainer` union is told apart by, so a switch or a comparison names one. */
export enum EXrayAssetContainer {
  /** A loose file, preserving its root so consumers can identify the winning overlay. */
  DIRECTORY = "directory",
  /** An entry inside the archive volume set at `path`. */
  ARCHIVE = "archive",
}

/** The physical container of a located asset. */
export type XrayAssetContainer =
  /** A loose file, preserving its root so consumers can identify the winning overlay. */
  | { kind: "directory"; root: string; relativePath: string }
  /** An entry inside the archive volume set at `path`. */
  | { kind: "archive"; path: string };

/** Asset category inferred from an X-Ray logical path's extension or recognized suffix. */
export enum EXrayAssetType {
  AI = "ai",
  ANM = "anm",
  C_FORM = "cForm",
  DDS = "dds",
  DM = "dm",
  EFD = "efd",
  ENV_MOD = "envMod",
  FOG_VOL = "fogVol",
  GAME = "game",
  GEOM = "geom",
  GEOM_X = "geomX",
  HOM = "hom",
  INI = "ini",
  LEVEL = "level",
  LIGHTS = "lights",
  LTX = "ltx",
  MISC = "misc",
  OGF = "ogf",
  OGG = "ogg",
  OGM = "ogm",
  OMF = "omf",
  PPE = "ppe",
  PS_STATIC = "psStatic",
  SND_STATIC = "sndStatic",
  SOM = "som",
  SCRIPT = "script",
  SEQ = "seq",
  SHADER = "shader",
  SPAWN = "spawn",
  THM = "thm",
  WALLMARKS = "wallmarks",
  DETAILS = "details",
  XR_PACK = "xrPack",
}

/** Every `EXrayAssetType` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type XrayAssetType = `${EXrayAssetType}`;

/** An X-Ray logical path: lower case, backslash separated, with no empty, `.` or `..` component. */
export type XrayLogicalPath = string;

/** How a caller's path is turned into mounts. */
export enum EXrayMountMode {
  /**
   * Treat the path as an installation when it declares one, as one volume when it is one, as a volume set when it
   * holds volumes, and as a complete root otherwise.
   */
  AUTO = "auto",
  /** Treat the path as a complete X-Ray root, ignoring any `fsgame.ltx` beside it. */
  DIRECTORY = "directory",
  /** Treat the path as one archive volume, or as every volume beneath a directory, and mount each on its own. */
  VOLUMES = "volumes",
  /** Require the path to declare an installation, and mount everything it declares. */
  INSTALLATION = "installation",
  /** Mount the nearest installation containing the path, searching upwards for `fsgame.ltx`. */
  CONTAINING_INSTALLATION = "containingInstallation",
}

/** Every `EXrayMountMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type XrayMountMode = `${EXrayMountMode}`;

/** Two files in one source claiming the same engine identity. */
export type XrayPathCollision = {
  /** Engine identity both files normalize to. */
  logicalPath: XrayLogicalPath;
  /** File the source resolves. */
  kept: string;
  /** File no lookup can reach, because `kept` already claims its identity. */
  unreachable: string;
};

/** Every `kind` the `XrayResolution` union is told apart by, so a switch or a comparison names one. */
export enum EXrayResolution {
  /** The reference itself resolved. */
  RESOLVED = "resolved",
  /** The reference did not resolve, but the fallback the caller offered did. */
  SUBSTITUTED = "substituted",
  /** Nothing resolved, across every step of the probe. */
  MISSING = "missing",
  /** There was nothing to search: the probe had no step, or no step selected a mounted source. */
  NO_SCOPE = "noScope",
  /** The reference could not be turned into a lookup at all, so none was attempted. */
  REJECTED = "rejected",
}

/** What one reference lookup came to. */
export type XrayResolution =
  /** The reference itself resolved. */
  | { kind: "resolved"; step: string; assets: Array<XrayAsset> }
  /** The reference did not resolve, but the fallback the caller offered did. */
  | { kind: "substituted"; step: string; fallback: string; assets: Array<XrayAsset> }
  /** Nothing resolved, across every step of the probe. */
  | { kind: "missing"; roots: Array<string> }
  /** There was nothing to search: the probe had no step, or no step selected a mounted source. */
  | { kind: "noScope" }
  /** The reference could not be turned into a lookup at all, so none was attempted. */
  | { kind: "rejected"; reason: string };

/** One place to read from, and how that place becomes mounts. */
export type XrayRoot = {
  /** Native host address, retained without rendering it as text. */
  path: string;
  /** How this path becomes mounts. `Auto` unless the caller says otherwise. */
  mode?: XrayMountMode;
};

/** The kind a probed path belongs to. */
export enum EXrayRootKind {
  /** The path declares an installation with `fsgame.ltx`. */
  INSTALLATION = "installation",
  /** The path is one archive volume, or a directory of them. */
  VOLUMES = "volumes",
  /** The path is a directory holding content an engine would load. */
  ROOT = "root",
  /** The path is a directory, but nothing beneath it looks like game data. */
  UNRECOGNIZED = "unrecognized",
  /** Nothing is there, or it cannot be read. */
  MISSING = "missing",
}

/** Every `EXrayRootKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type XrayRootKind = `${EXrayRootKind}`;

/** What a path turns out to be when planned, and why. */
export type XrayRootProbe = {
  /** What the path is, as planning sees it. */
  kind: XrayRootKind;
  /** Which of the well-known entries sit directly beneath the path. */
  evidence: Array<string>;
  /** How many sources the path plans into, or zero when it plans into none. */
  mounts: number;
};

/** Everywhere a caller wants read: an optional subject asset, then ordered roots. */
export type XrayRoots = {
  /** Native asset address whose own X-Ray root and installation are searched first, when the read is centred on one. */
  asset: string | null;
  /** Roots searched after the asset's own, in the order given. */
  roots: Array<XrayRoot>;
};

/** The storage kind backing a mount. */
export enum EXraySourceKind {
  /** Loose files under a directory. */
  DIRECTORY = "directory",
  /** Entries inside a set of `.db` archive volumes. */
  ARCHIVE = "archive",
}

/** Every `EXraySourceKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type XraySourceKind = `${EXraySourceKind}`;
