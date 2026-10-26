// Auto-generated rust bindings. Do not edit it manually.

import { AssetSource, AssetTextureDescriptor, FileStamp, SessionId } from "#/types/xrf-app-core";
import { JobOutcome } from "#/types/xrf-job";
import { XrayMaterialDescriptor } from "#/types/xrf-material";
import { TextureEncodingFormat, TextureEncodingQuality } from "#/types/xrf-texture";
import { XrayAsset, XrayRoots } from "#/types/xrf-vfs";

/** Rebuilding a texture from a source image, as its descriptor says to. */
export const TEXTURES_JOB_BUILD = "textures.build" as const;

/** Weighing every candidate format against one texture. */
export const TEXTURES_JOB_COMPARE_ENCODINGS = "textures.compare-encodings" as const;

/** Generating a bump pair from a height map. */
export const TEXTURES_JOB_MAKE_BUMP = "textures.make-bump" as const;

/** Saving a node's descriptor, and its texture with it. */
export const TEXTURES_JOB_SAVE = "textures.save" as const;

/** What a tree shows on a texture before anyone opens it, read from its descriptor alone. */
export type TextureBadges = {
  /** Both halves of the declared pair resolved to the files the declaration names. */
  isBumped: boolean;
  /** The bump shader path is taken but a half is a dummy or absent, so the surface is not what was authored. */
  isDegraded: boolean;
  /** The descriptor's texture type makes `LoadTHM` skip it whole, bump declaration included. */
  isEngineSkipped: boolean;
  /** A detail texture is named and one of the two flags that switch it on is set. */
  isDetailAssociated: boolean;
  /** A `.thm` sits there and does not parse as one. */
  isUnreadable: boolean;
};

/** What a tree is listing: the roots and how it addresses them, restored after a frontend reload. */
export type TextureBrowse = {
  roots: XrayRoots;
  mode: TextureCatalogMode;
};

/** One recipe field the descriptor asks for that this build does not carry out. */
export type TextureBuildOmissionReport = {
  /** The descriptor field this sits beside, under the name the SDK gives it. */
  field: string;
  /** Why the build does not do it, in words a person deciding whether to rebuild can act on. */
  reason: string;
};

/** What a rebuilt texture came to. */
export type TextureBuildOutcome = {
  /** Whether the texture was written or the run stopped before it started. */
  outcome: JobOutcome;
  destination: string;
  /**
   * The `#` half a bump descriptor writes beside the texture, the compression error and the height; absent for any
   * other descriptor.
   */
  companion: string | null;
  /** Mean gloss of a bump descriptor's pair, in `0..=1`; absent for any other descriptor. */
  glossPower: number | null;
  /** Whether that gloss is too dark for the surface to show a specular response worth having. */
  isGlossTooDark: boolean;
  /** Size of the source, which the descriptor's own width and height are refreshed from. */
  width: number;
  height: number;
  /** Levels written, counting the base. */
  mipmapLevels: number;
  /** Recipe fields the descriptor asked for and the build did not carry out. */
  omissions: Array<TextureBuildOmissionReport>;
};

/** Every `kind` the `TextureBumpGloss` union is told apart by, so a switch or a comparison names one. */
export enum ETextureBumpGloss {
  /** The height map's own alpha, where the SDK's bump source carries its gloss. */
  HEIGHT_ALPHA = "heightAlpha",
  /** One level for the whole surface, from 0 to 1, for a surface authored without a gloss. */
  CONSTANT = "constant",
  /** Path of a gloss mask, averaged across its colour channels. */
  MASK = "mask",
}

/** Where a generated bump pair takes its gloss from. */
export type TextureBumpGloss =
  /** The height map's own alpha, where the SDK's bump source carries its gloss. */
  | { kind: "heightAlpha" }
  /** One level for the whole surface, from 0 to 1, for a surface authored without a gloss. */
  | { kind: "constant"; level: number | null }
  /** Path of a gloss mask, averaged across its colour channels. */
  | { kind: "mask"; path: string };

/** The two references a declaration binds, which is what folds a pair under the texture that declares it. */
export type TextureBumpPair = {
  bump: string;
  companion: string;
};

/** Every texture the roots hold, once per reference, in reference order. */
export type TextureCatalog = {
  /** How this listing was made, which decides what its rows are addressed by and whether a sweep can badge them. */
  mode: TextureCatalogMode;
  /** The roots the catalog was listed from, so a later read searches what the listing searched. */
  roots: XrayRoots;
  /** A `textures.ltx` the roots hold, or `None`. Its declarations are not read, so a surface says where it matters. */
  texturesLtx: XrayAsset | null;
  entries: Array<TextureEntry>;
  /** `.dds` files the roots hold outside `textures\`, which no engine reference names and this catalog leaves out. */
  outsideTexturesCount: number;
  /** Loose files whose path is not Unicode, which the page could not name back and this catalog leaves out. */
  unnamedCount: number;
};

/** How a listing addressed what it found. */
export enum ETextureCatalogMode {
  /** The game tree: listed by engine reference, archives included, files outside `textures\` counted not listed. */
  ROOTS = "roots",
  /** A plain directory: every `.dds` under it, addressed by its own path. */
  LOOSE_DIRECTORY = "looseDirectory",
}

/** Every `ETextureCatalogMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TextureCatalogMode = `${ETextureCatalogMode}`;

/** Everything the inspection panels say about one texture, resolved in one call. */
export type TextureDescription = {
  source: TextureSource;
  /** What to call this texture on screen: its engine reference, or a standalone file's own stem. */
  reference: string;
  /** The roots the description was resolved in, so a later read searches what this searched. */
  roots: XrayRoots;
  /** The `.dds` the reference resolves to, or `None` for a descriptor with no texture. */
  texture: XrayAsset | null;
  /**
   * Where the texture's pixels are read from: an asset of `roots`, or a standalone file by its path, which nothing
   * mounts; `None` for a descriptor with no texture.
   */
  textureSource: AssetSource | null;
  /** What the base texture file is, when it is located and its bytes can be reached. */
  base: AssetTextureDescriptor | null;
  /** What the renderer would build for this texture, or `None` for a file outside every tree. */
  material: XrayMaterialDescriptor | null;
  /** What the bound bump file is, when the material binds one and its bytes can be reached. */
  bump: AssetTextureDescriptor | null;
  /** What the bound bump companion file is, on the same terms. */
  companion: AssetTextureDescriptor | null;
  /** Whether a `.thm` was located and parsed, so the editor edits one rather than authoring it. */
  hasDescriptor: boolean;
  /** The descriptor's editable fields, or the SDK's defaults a descriptor authored for this texture starts from. */
  form: TextureDescriptorForm;
  /** Where an edit of this texture would write, absent for a texture served out of an archive. */
  targets: TextureEditTargets | null;
  /** The pair the bump generator writes for this texture, as the references a descriptor names. */
  generatedBump: TextureBumpPair;
};

/** The descriptor fields the editor owns, read off a `.thm` and written back onto one. */
export type TextureDescriptorForm = {
  /** The thumbnail's texture type, which is the gate `LoadTHM` reads before anything else. */
  textureType: number;
  bumpMode: number;
  /** Bump texture path without extension, engine-style with backslashes. Empty when unused. */
  bumpName: string;
  /** Detail texture path on the same terms. */
  detailName: string;
  detailScale: number | null;
  /** The whole `STextureParams` flag word, named bits and unnamed alike. */
  flags: number;
  format: number;
  mipFilter: number;
  borderColor: number;
  fadeColor: number;
  fadeAmount: number;
  /** Mip level the converter starts fading from. */
  fadeDelay: number;
  material: number;
  materialWeight: number | null;
  extNormalMapName: string;
  /** Read by the bump generator and by nothing at runtime. */
  virtualHeight: number | null;
  /** What the descriptor claims its texture measures, which the DDS beside it is the authority on. */
  width: number;
  height: number;
};

/** The descriptor half of a save, which every save writes. */
export type TextureDescriptorSave = {
  target: TextureSaveTarget;
  form: TextureDescriptorForm;
};

/** Where an edit of one texture would write, and what was there when the editor read it. */
export type TextureEditTargets = {
  /** The `.thm` to write, whether or not one is there yet. */
  descriptor: TextureSaveTarget;
  /** The `.dds` to replace when a re-encode is saved. */
  texture: TextureSaveTarget;
};

/** Every candidate weighed against one texture, with the texture itself for a baseline. */
export type TextureEncodingComparison = {
  /** The comparison holding the encodes, which a save names to claim one. */
  sessionId: SessionId;
  source: TextureSource;
  roots: XrayRoots;
  /** Whether every candidate was weighed or the run stopped because it was asked to. */
  outcome: JobOutcome;
  /** What to call the texture the encodes were made from. */
  reference: string;
  current: TextureEncodingCurrent;
  /** The candidates weighed, in the order `DdsEncodeCandidate::ALL` lists them, cheapest first. */
  candidates: Array<TextureEncodingReport>;
};

/** The texture as it stands, for the row a comparison is read against. */
export type TextureEncodingCurrent = {
  /** Format name from the file's own header, which is not always one of the five candidates. */
  label: string;
  fileBytes: number;
  gpuBytes: number;
  width: number;
  height: number;
  mipmapLevels: number;
};

/** What one candidate cost and what it lost, weighed against the texture as it is now. */
export type TextureEncodingReport = {
  format: TextureEncodingFormat;
  /** The format's name with the DXT name the descriptor and the SDK use for it. */
  label: string;
  /** Bytes on disk, header included. */
  fileBytes: number;
  /** Bytes once uploaded, which is the whole mip chain without the header. */
  gpuBytes: number;
  encodeDuration: number;
  /** Peak signal-to-noise ratio, absent where nothing was lost. */
  psnr: number | null;
  /** Root mean square error per channel, in the eight-bit units the pixels are stored in. */
  channelRmse: [number | null, number | null, number | null, number | null];
  /** What the renderers together make of the format, as a sentence a badge can show. */
  supportSummary: string;
  compatibility: Array<TextureRendererSupport>;
};

/**
 * The texture half of a save: one candidate a held comparison encoded, written over the texture that comparison was
 * made of, which it names itself.
 */
export type TextureEncodingSave = {
  /** The comparison holding the candidate. */
  sessionId: SessionId;
  format: TextureEncodingFormat;
  /** The stamp the editor read on the texture, or `None` where it saw none. */
  expected: FileStamp | null;
};

/** One texture name and the files the roots hold for it. */
export type TextureEntry = {
  /** What to call this row: an engine reference such as `ston\ston_beton05`, or a loose file's path below its root. */
  reference: string;
  /** How to open this row, which is the address the describe and every write take. */
  source: TextureSource;
  role: TextureRole;
  /** The `.dds` the roots answer for this reference, winner first. */
  texture: XrayAsset | null;
  /** The `.thm` the roots answer for this reference, winner first. */
  descriptor: XrayAsset | null;
};

/** One bit of the `STextureParams` flag word. */
export type TextureFlagEntry = {
  /** The bit, as a mask rather than an index, because the SDK's own bits are not contiguous. */
  bit: number;
  label: string;
};

/** What one renderer makes of a format. */
export enum ETextureFormatSupport {
  SUPPORTED = "supported",
  UNSUPPORTED = "unsupported",
  /** A path nobody has read, not a refusal. */
  UNVERIFIED = "unverified",
}

/** Every `ETextureFormatSupport` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TextureFormatSupport = `${ETextureFormatSupport}`;

/** What a generated pair came to. */
export type TextureMakeBumpOutcome = {
  /** Whether the pair was written or the run stopped because it was asked to. */
  outcome: JobOutcome;
  /** The normals and gloss, written as `<bump>.dds`. */
  bump: string;
  /** The compression error and the height, written as `<bump>#.dds`. */
  companion: string;
  /** Mean gloss over the whole surface, in `0..=1`. */
  glossPower: number | null;
  /** Whether the gloss is too dark for the surface to show a specular response worth having. */
  isGlossTooDark: boolean;
};

/** One descriptor's contribution to the tree: its badges, and the pair it names so both halves fold under it. */
export type TextureMaterialSummary = {
  /** The reference of the texture the descriptor describes. */
  reference: string;
  /** The pair the engine will try to bind, when the declaration is one it reads. */
  bump: TextureBumpPair | null;
  badges: TextureBadges;
};

/** One renderer's answer about a format. */
export type TextureRendererSupport = {
  renderer: string;
  support: TextureFormatSupport;
};

/** What a texture name is by convention, before any descriptor has been read. */
export enum ETextureRole {
  /** A texture a mesh or a level binds by name. */
  TEXTURE = "texture",
  /** The first half of a bump pair: packed normal and gloss. */
  BUMP = "bump",
  /** The second half of a bump pair: packed error and height. */
  BUMP_COMPANION = "bumpCompanion",
}

/** Every `ETextureRole` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TextureRole = `${ETextureRole}`;

/** What a save left on disk. */
export type TextureSaveOutcome = {
  /** Whether the save wrote what it was asked to or stopped because it was asked to. */
  outcome: JobOutcome;
  /** The files written, in the order they were written. */
  written: Array<string>;
};

/** One file a write addresses, and what was there when the editor read it. */
export type TextureSaveTarget = {
  /** Absolute path of the file to replace or create. */
  path: string;
  /** The stamp the editor read there, or `None` for a file it is creating. */
  expected: FileStamp | null;
};

/** Every `kind` the `TextureSource` union is told apart by, so a switch or a comparison names one. */
export enum ETextureSource {
  /** A loose `.dds` or `.thm` on disk, named by its filesystem path. */
  FILE = "file",
  /** A texture of the roots, loose or archived, named by its engine reference such as `ston\ston_beton05`. */
  ASSET = "asset",
}

/** Where a texture is named from. */
export type TextureSource =
  /** A loose `.dds` or `.thm` on disk, named by its filesystem path. */
  | { kind: "file"; path: string }
  /** A texture of the roots, loose or archived, named by its engine reference such as `ston\ston_beton05`. */
  | { kind: "asset"; reference: string };

/** Which of the engine's three readings of a texture's alpha the body is drawn with. */
export enum ETextureSurfaceAlpha {
  /** Not read at all, which is what the plain deferred base shader does. */
  IGNORED = "ignored",
  /** Clipped against `def_aref`, which is what every `_aref` shader does. */
  CUT_OUT = "cutOut",
  /** Composited over what is behind it, which the deferred pass never does. */
  BLENDED = "blended",
}

/** Every `ETextureSurfaceAlpha` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TextureSurfaceAlpha = `${ETextureSurfaceAlpha}`;

/** The body a texture is laid on to be looked at. */
export enum ETextureSurfaceShape {
  /** Flat and face on, where the decode is read most directly and tiling is judged. */
  PLANE = "plane",
  /** Curved, so the normal sweeps every grazing angle a wrong tangent sign shows up at. */
  SPHERE = "sphere",
  /** Edged, where a seam and the wrap of a tiling texture meet. */
  CUBE = "cube",
}

/** Every `ETextureSurfaceShape` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TextureSurfaceShape = `${ETextureSurfaceShape}`;

/** Every named value the descriptor form's numeric fields can take. */
export type TextureVocabulary = {
  /** The gate `LoadTHM` reads before anything else. */
  textureTypes: Array<TextureVocabularyEntry>;
  formats: Array<TextureVocabularyEntry>;
  /** Fifteen values, of which fourteen are kernels and `Advanced` is the SDK's own chain. */
  mipFilters: Array<TextureVocabularyEntry>;
  materials: Array<TextureVocabularyEntry>;
  bumpModes: Array<TextureVocabularyEntry>;
  /** The twelve bits the SDK names, in bit order. A word may carry others, and those have no name to show. */
  flags: Array<TextureFlagEntry>;
  /** The bump mode that makes the engine bind a pair. */
  bumpModeUse: number;
  /** The mean gloss below which a generated bump shows almost no specular response. */
  minimumGlossPower: number | null;
};

/** One value a descriptor field can take, under the name the SDK gives it. */
export type TextureVocabularyEntry = {
  /** The number stored in the file. */
  value: number;
  /** The SDK's own identifier for it, which is the name an author of a `.thm` would recognise. */
  label: string;
};

/** What a texture rebuild was asked to do. */
export type TexturesBuildRequest = {
  /** Path of the `.dds` to write, which is the file beside the descriptor. */
  destination: string;
  /**
   * Path of the image to encode, of whatever kind `image` decodes; for a bump descriptor, the height in its colour and
   * the gloss in its alpha.
   */
  source: string;
  /** The descriptor to read as a recipe, as the editor currently has it rather than as it is on disk. */
  descriptor: TextureDescriptorForm;
  /**
   * Path of the texture a bump descriptor names as its external normal map, whose own `.thm` beside it says what it
   * is; absent for any other.
   */
  normalMap: string | null;
  quality: TextureEncodingQuality;
};

/** What a format comparison was asked to weigh. */
export type TexturesCompareRequest = {
  sessionId: SessionId;
  /** The texture to re-encode, named the way `describe` names one. */
  source: TextureSource;
  roots: XrayRoots;
  /** Kernel the chain is reduced with, by its SDK name, or `None` to weigh the base level alone. */
  mipFilter: string | null;
  quality: TextureEncodingQuality;
};

/**
 * What a bump pair generation was asked to do.
 *
 * The height source is required and everything else refines it, so a caller with only a height map still gets a
 * pair. The SDK's generator reads the gloss from that one source's alpha; a constant or a mask stands in for a source
 * without one.
 */
export type TexturesMakeBumpRequest = {
  /** Path of the texture the pair belongs to, without the `_bump` suffix or an extension. */
  destination: string;
  /** Path of the image the relief is read from, averaged across its colour channels. */
  height: string;
  gloss: TextureBumpGloss;
  /** Path of a normal map to use instead of deriving one from the height, of the same size. */
  normalMap: string | null;
  /** `bump_virtual_height` of the descriptor, read here and nowhere at runtime; the SDK's default when absent. */
  virtualHeight: number | null;
  /** Kernel the pair's chain is reduced with, by its SDK name. */
  mipFilter: string | null;
  quality: TextureEncodingQuality;
};

/**
 * What one node's save was asked to write: its descriptor every time, and a re-encoded texture with it when one was
 * chosen, so a written texture is never left beside a descriptor naming its old format.
 */
export type TexturesSaveRequest = {
  descriptor: TextureDescriptorSave;
  texture: TextureEncodingSave | null;
};

/** A texture laid on a body, and how it is laid there. */
export type TexturesShowRequest = {
  source: TextureSource;
  /** The roots the texture is resolved in, as its description was. */
  roots: XrayRoots;
  shape: TextureSurfaceShape;
  /** How many times the texture repeats across the body, which is how a tiling seam becomes visible. */
  tiling: number | null;
  alpha: TextureSurfaceAlpha;
  /** Width over height of the texture, which the plane is stretched to. */
  aspect: number | null;
};
