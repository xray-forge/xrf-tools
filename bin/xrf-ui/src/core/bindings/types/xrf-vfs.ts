// Auto-generated rust bindings. Do not edit it manually.

/**
 * One asset a mount resolved: its engine identity plus the container it came out of.
 *
 * Owned rather than borrowed, so it can be stored, sorted or sent over IPC — which is what an editor that mounts and
 * writes needs, and why nothing borrowed reaches past this crate.
 */
export type XrayAsset = {
  /** Lower-case, backslash-separated engine identity, including the mount's logical base. */
  logicalPath: XrayLogicalPath;
  /** Physical container reported by the source that resolved the asset. */
  container: XrayAssetContainer;
};

/**
 * The physical container of a located asset.
 *
 * Separate variants prevent callers from treating an archived entry as a loose file with a usable filesystem path.
 */
export type XrayAssetContainer =
  /** A loose file, preserving its root so consumers can identify the winning overlay. */
  | { kind: "directory"; root: string; relativePath: string }
  /** An entry inside the archive volume set at `path`. */
  | { kind: "archive"; path: string };

/**
 * Asset category inferred from an X-Ray logical path's extension or recognized suffix.
 *
 * Serialized so a consumer can name the kind it wants without the crate growing a command per kind, which is the same
 * reason [`XrayAssetType::get_rules`] is a table rather than a method each.
 */
export type XrayAssetType =
  | "ai"
  | "anm"
  | "cForm"
  | "dds"
  | "dm"
  | "efd"
  | "envMod"
  | "fogVol"
  | "game"
  | "geom"
  | "geomX"
  | "hom"
  | "ini"
  | "level"
  | "lights"
  | "ltx"
  | "misc"
  | "ogf"
  | "ogg"
  | "ogm"
  | "omf"
  | "ppe"
  | "psStatic"
  | "sndStatic"
  | "script"
  | "seq"
  | "shader"
  | "spawn"
  | "thm"
  | "wallmarks"
  | "details"
  | "xrPack";

/**
 * An X-Ray logical path: lower case, backslash separated, with no empty, `.` or `..` component.
 *
 * Being separator-explicit is what makes it portable: it splits on `\` itself rather than deferring to
 * `std::path`, so `parent` and `file_name` answer the same on Linux as on Windows, where a `std::path::Path`
 * would treat the whole thing as one component.
 *
 * Serialized and typed transparently as its string form, so an engine path crosses IPC as the text the engine uses.
 */
export type XrayLogicalPath = string;

/**
 * How a caller's path is turned into mounts.
 *
 * One vocabulary for every tool, so `--source` means the same thing everywhere. Each variant maps onto an
 * [`XrayMountPlan`] constructor; this exists so a command surface, an app setting, and an editor can all name
 * the choice rather than each re-deriving it.
 */
export type XrayMountMode =
  /**
   * Treat the path as an installation when it declares one, as a volume set when it holds volumes, and as a complete
   * root otherwise.
   */
  | "auto"
  /** Treat the path as a complete X-Ray root, ignoring any `fsgame.ltx` beside it. */
  | "directory"
  /** Require the path to declare an installation, and mount everything it declares. */
  | "installation"
  /** Mount the nearest installation containing the path, searching upwards for `fsgame.ltx`. */
  | "containingInstallation";

/**
 * What one reference lookup came to.
 *
 * A fact about a lookup, not about the kind of thing looked up: a texture, a motion set and a level asset all end in
 * one of these states, so a consumer renders one shape and a domain crate pairs the outcome with its own reference
 * identity rather than defining its own vocabulary.
 *
 * A missing asset is a state rather than an error, because it is one in the engine too — `Missing` carries where the
 * probe looked so a report can say that instead of only that nothing was found.
 */
export type XrayResolution =
  /**
   * The reference itself resolved.
   *
   * `assets` is never empty, and holds more than one entry only for a mask — a motion reference may name a set.
   */
  | { kind: "resolved"; step: string; assets: Array<XrayAsset> }
  /**
   * The reference did not resolve, but the fallback the caller offered did.
   *
   * Substitution is engine behavior a caller opts into per kind, so the fallback reference travels back: reporting the
   * asset alone would show a located texture while hiding that it is not the requested one.
   */
  | { kind: "substituted"; step: string; fallback: string; assets: Array<XrayAsset> }
  /**
   * Nothing resolved, across every step of the probe.
   *
   * `roots` is every source searched, in probe order and without duplicates.
   */
  | { kind: "missing"; roots: Array<string> }
  /**
   * There was nothing to search: the probe had no step, or no step selected a mounted source.
   *
   * Distinct from `Missing` because the question could not be asked rather than the answer being no, which is
   * the difference between an unconfigured project and an absent asset.
   */
  | { kind: "noScope" }
  /**
   * The reference could not be turned into a lookup at all, so none was attempted.
   *
   * Engine text is untrusted: a mesh header may hold a name no logical path can be made of. Folding that into `Missing`
   * would report a garbage reference as an absent asset, and substituting for it would report it as a present one.
   */
  | { kind: "rejected"; reason: string };

/** One root a world is assembled from, and how that root becomes mounts. */
export type XrayWorldRoot = {
  path: string;
  /** How this path becomes mounts. `Auto` unless the caller says otherwise. */
  mode?: XrayMountMode;
};

/**
 * Where a world is: an optional subject asset, then ordered roots.
 *
 * The one way every surface names a world, so `--source` on a command, a setting in the app, and an
 * editor session all say the same thing. What sits *inside* the world is a separate question that
 * stays with each domain — a dialog layout and a translations layout disagree, and a spawn file has
 * no answer at all.
 *
 * Several roots is layering, which is how modding actually works: a loose gamedata tree in front of
 * an installation. Search order is declaration order, and the first mount holding a path wins.
 *
 * Callers do not assemble mounts from this themselves. They hand it to whatever owns mounting and
 * receive a world or a probe, so there is one place that decides what a spec means.
 */
export type XrayWorldSpec = {
  /**
   * Asset whose own X-Ray root and installation are searched first, when the world is centred on one.
   *
   * This is what finds a texture shipped beside a model rather than in the shared tree.
   */
  asset: string | null;
  /** Roots searched after the asset's own, in the order given. */
  roots: Array<XrayWorldRoot>;
};
