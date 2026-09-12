// Auto-generated rust bindings. Do not edit it manually.

import { ArchiveProject, ArchiveReadPolicy } from "@/core/bindings/types/xrf-archive";
import { SpawnHeaderChunk } from "@/core/bindings/types/xrf-db";
import { DialogProjectMode } from "@/core/bindings/types/xrf-dialog";
import { JobOutcome, JobProgress } from "@/core/bindings/types/xrf-job";
import { LtxAnchoredFinding, LtxFileStructure, LtxFileText, LtxInventory } from "@/core/bindings/types/xrf-ltx-inspect";
import { XrayMaterialDescriptor, XraySurfaceDescriptor } from "@/core/bindings/types/xrf-material";
import { ArchivePackConfig, ArchivePatchConfig } from "@/core/bindings/types/xrf-pack";
import { InventorySpriteDescriptor } from "@/core/bindings/types/xrf-texture";
import {
  TranslationBuildLanguageSummary,
  TranslationParseCensus,
  TranslationProjectDescriptor,
  TranslationProjectMode,
  TranslationVerifyLanguageSummary,
} from "@/core/bindings/types/xrf-translation";
import { XrayAsset, XrayAssetContainer, XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { VisualDependencies, VisualDescription } from "@/core/bindings/types/xrf-visual";

/**
 * What the explorer has open: a set of `.db` volumes, or a whole mounted world.
 *
 * The two are not folded into one shape. A volume set can say which volume an entry sits in, at what offset, with
 * which recorded CRC; a world can say which copy of an engine path wins and what that decision hides. A single
 * descriptor covering both would have a loose file claiming a volume position, which is the fiction this split
 * exists to avoid.
 *
 * Every difference between them is answered here, so a command stays an adapter and a reader asking how browsing an
 * installation differs from browsing a volume set opens one file.
 */
export type ArchiveSubject =
  /** The volumes at one path, merged into a single name table. */
  | { kind: "volumes"; project: ArchiveProject }
  /** A game folder read as the engine mounts it, archives and loose tree together. */
  | { kind: "world"; world: ArchiveWorld };

/**
 * One mounted world the explorer browses: an installation, or any tree read as the engine would read it.
 *
 * The other subject of the same explorer answers for one volume set and nothing else. Pointing that at a game folder
 * lists the archives and silently omits the loose `gamedata` tree in front of them, so it shows the archived payload
 * for files the engine would serve from disk. This one answers the other question: which copy actually wins, and what
 * that decision hides.
 *
 * A listing rather than a VFS. The mounts live in the application's one [`crate::core::assets::AssetMountState`],
 * where every other surface's reads already go and where mounting one installation twice costs one index; what the
 * session owns is the answer it published.
 */
export type ArchiveWorld = {
  /** How the world was opened, so every later read of it addresses exactly these mounts. */
  roots: XrayRoots;
  /** Sources searched, highest priority first, as each one names itself. */
  mounts: Array<string>;
  /** Winning entries, one per engine path, ordered by that path. */
  files: Array<ArchiveWorldEntry>;
  readPolicy: ArchiveReadPolicy;
  /** Unpacked bytes of the winning entries. What is shadowed is not counted; it is not what the engine would load. */
  sizeReal: number;
  /** Engine paths this world answers with more than one copy for. */
  shadowedCount: number;
};

/**
 * One file of a mounted world, as the explorer lists it.
 *
 * Shaped like [`xrf_archive::ArchiveFileDescriptor`] where the two can agree — a `name` and a `size_real` — because
 * the tree, the filter and the preview gate above them need nothing else, and giving each subject its own spelling of
 * those two would fork every one of those surfaces.
 *
 * Where they cannot agree, this says less rather than inventing something. A loose file has no volume position, no
 * stored size and no recorded CRC, so nothing here claims one; what it has instead is the copies it stands in front
 * of, which a volume set has no way to express.
 */
export type ArchiveWorldEntry = {
  /** Engine identity: lower-case and backslash separated, which is what every read of this world is addressed by. */
  name: string;
  /** Where the winning copy physically sits — a file on disk, or an entry of a volume. */
  container: XrayAssetContainer;
  /** Payload bytes once unpacked. */
  sizeReal: number;
  /** Copies of this engine path no lookup reaches, in mount priority order behind the winner. */
  shadowed: Array<XrayAssetContainer>;
};

/**
 * Directory extraction request for whichever subject the explorer has open.
 *
 * One shape for both: the two differ in where the bytes come from, never in what a person asked for.
 */
export type ArchivesExtractRequest = {
  sessionId: SessionId;
  /** Directory inside the opened tree to extract. An empty prefix means everything. */
  prefix: string;
  /** Directory to write the contents into. */
  destination: string;
};

/** Archive packing request. */
export type ArchivesPackRequest = {
  /** What to pack and how. */
  config: ArchivePackConfig;
  /** Whether an existing output may be overwritten. */
  isForced: boolean;
};

/** Shared request for archive comparison and patch publication. */
export type ArchivesPatchRequest = {
  /** What to compare and where to publish the difference. */
  config: ArchivePatchConfig;
  /** Whether an existing output may be overwritten. Ignored by a comparison, which writes nothing. */
  isForced: boolean;
  /** Whether a checksum match should be proven by comparing the payloads themselves. */
  isVerifyingPayload: boolean;
};

/** Archive unpacking request. */
export type ArchivesUnpackRequest = {
  /** Archive or directory of archives to read. */
  from: string;
  /** Directory to write the contents into. */
  destination: string;
};

/**
 * What a texture file is, once it has been located.
 *
 * Reported by the command that resolved the reference rather than derived by the frontend: the facts all come from a
 * DDS header, `xrf-dds` already reads one, and a renderer-side reimplementation would name the same formats
 * differently than the `verify-ogf` census does.
 */
export type AssetTextureDescriptor = {
  /** Bytes the file occupies, which is also what a renderer uploads for a block-compressed texture. */
  size: number;
  /**
   * Header facts, absent when the bytes are not a readable DDS.
   *
   * Nested rather than four independent options, so a partially known shape cannot be described: either the header
   * parsed and every field is from it, or it did not and the size is all that is known.
   */
  shape: AssetTextureShape | null;
};

/** Pixel layout a DDS header declares. */
export type AssetTextureShape = {
  width: number;
  height: number;
  /**
   * Levels the file carries, one meaning no mip chain at all.
   *
   * Load bearing rather than trivia: a texture without mips has to be sampled with a linear filter or webgl renders it
   * black, and 1,805 of Anomaly's 2,197 distinct textures ship without one.
   */
  mipmapLevels: number;
  /** Format name from [`DdsMetadata::get_format_label`], so the viewer and the sweep agree on what a file is. */
  format: string;
};

/**
 * What a sound is, once it has been located.
 *
 * Every field is optional because there are two independent ways to know less than everything: bytes that are not a
 * readable ogg at all, and a perfectly good ogg carrying no X-Ray comment. Reporting a zero for either would make an
 * unreadable file indistinguishable from a silent one.
 */
export type AudioDescriptor = {
  /** Absent when the bytes carry no readable stream header. */
  channels: number | null;
  /** Absent when the bytes carry no readable stream header. */
  sampleRate: number | null;
  /** Absent for a sound carrying no recognized X-Ray comment, where the engine uses its own defaults. */
  parameters: AudioSourceParameters | null;
};

/** The X-Ray source parameters carried in a sound's first vorbis comment. */
export type AudioSourceParameters = {
  minDistance: number | null;
  maxDistance: number | null;
  baseVolume: number | null;
  gameType: number;
  maxAiDistance: number | null;
};

/**
 * One config as the authored view renders it.
 *
 * Text and structure travel together but stay separate records: the text is what a person edits and the structure is
 * what only the parser knows, and a future edit replaces one without invalidating the shape of the other.
 */
export type ConfigsDocument = {
  text: LtxFileText;
  structure: LtxFileStructure;
  /** What is wrong with this file itself: it will not parse, or an `#include` reached nothing. */
  findings: Array<LtxAnchoredFinding>;
};

/** What a config formatting run, or a check of one, was asked to do. */
export type ConfigsFormatRequest = {
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
};

/** What opening a configs project for browsing was asked to do. */
export type ConfigsOpenRequest = {
  sessionId: SessionId;
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
  /** Whether to resolve with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
};

/** What one open of the configs explorer answers with. */
export type ConfigsProjectDescriptor = {
  /** Identity every later read is addressed by. */
  sessionId: SessionId;
  /** The trees this project searched, as the backend resolved them, so a reload restores the same open. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
  /** Whether configs resolve under the Monolith/Anomaly patch dialect. */
  isDltx: boolean;
  /** Host path the project reports itself at, for a crumb that names something a person recognises. */
  root: string;
  /** Every config the project holds, and what each one is to it. */
  inventory: LtxInventory;
  /**
   * Section schemes the project's `*.scheme.ltx` files declare, by name.
   *
   * Sent once with the open rather than per document: a tree declares tens of them and every structure read joins
   * against the same set.
   */
  declaredSchemes: Array<string>;
};

/** Which config of which open a reader wants. */
export type ConfigsReadDocumentRequest = {
  /** The open this read is addressed to; a read naming a replaced one is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the config to read. */
  path: string;
};

/** Which sections of a resolved root a page wants. */
export type ConfigsReadSectionsRequest = {
  /** The open this read is addressed to; a read naming a replaced one is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the entry point the sections belong to. */
  entry: string;
  /** Sections to read, as the index named them. */
  names: Array<string>;
};

/** Which resolved root a reader wants, of which open. */
export type ConfigsResolvedRequest = {
  /** The open this read is addressed to; a read naming a replaced one is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the entry point to resolve. */
  entry: string;
};

/** Which section of a resolved root a reader wants explained. */
export type ConfigsSectionRequest = {
  /** The open this read is addressed to; a read naming a replaced one is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the entry point the section belongs to. */
  entry: string;
  /** The section to explain, as the index named it. */
  section: string;
};

/** What a config verification was asked to do. */
export type ConfigsVerifyRequest = {
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
  /** Whether to resolve with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
};

/** What opening a dialogs project was asked to do. */
export type DialogsOpenRequest = {
  sessionId: SessionId;
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** How much of the project to read. */
  mode: DialogProjectMode;
  /** Scope holding the dialog files, or nothing for all of them. */
  dialogsPrefix: string | null;
  /** Scope holding the string tables, or nothing for all of them. */
  translationsPrefix: string | null;
};

/** Identifies a dialog within one committed project. */
export type DialogsReadRequest = {
  sessionId: SessionId;
  logicalPath: string;
  id: string;
  language: string | null;
};

export type EquipmentSpriteMetadata = {
  path: string;
  name: string;
  systemLtxPath: string;
  /** Whether these descriptors came out of a DLTX-resolved config tree. */
  isDltx: boolean;
  equipmentDescriptors: Array<InventorySpriteDescriptor>;
};

/** One check's verdict, as the desktop surface shows it. */
export type GamedataCheckSummary = {
  /** The check that ran, spelled as the command line spells it. */
  check: string;
  /** `passed`, `failed`, `incomplete`, or `skipped`. */
  status: string;
  /** The check's own one-line verdict. */
  summary: string;
  findings: number;
  /** How long this check took, where it measured itself. */
  duration: number | null;
};

/** What a verification was asked to do. */
export type GamedataVerifyRequest = {
  /** Gamedata root to verify. */
  root: string;
  /** Checks to run, or nothing for every one this build knows. */
  checks: Array<string> | null;
  /** Whether a check that would warn should fail instead. */
  isStrict: boolean;
};

/** What a whole verification reports back to the desktop surface. */
export type GamedataVerifySummary = {
  /**
   * Whether every selected check finished, or the run stopped before or inside a check.
   *
   * A stopped run's checks are real verdicts; its silence about the rest is not one.
   */
  outcome: JobOutcome;
  /** The aggregate verdict over the checks that ran. */
  status: string;
  checks: Array<GamedataCheckSummary>;
  duration: number;
};

/**
 * How a job that is no longer running ended.
 *
 * Wider than `xrf_job::JobOutcome` on purpose: that one is what an operation reports about its own work, and an
 * operation that failed reports nothing at all — the failure travels as the command's error. The registry watches
 * from outside and has to describe that case too, or a job that blew up would sit in the listing looking finished.
 */
export type JobConclusion = "completed" | "cancelled" | "failed";

/**
 * One job as the listing describes it, running or recently finished.
 *
 * One shape for both rather than two, because the panel showing them shows one list: a job crossing from running to
 * finished should change its fields, not its type. `conclusion` is what separates the halves.
 */
export type JobDescription = {
  id: string;
  /** What kind of work this is, as the command that started it named itself. */
  kind: JobKind;
  /** What this job holds exclusively, so a refused start can be explained by pointing at the job that refused it. */
  leaseKeys: Array<string>;
  /**
   * What the job was asked to do, as the command that started it described itself.
   *
   * JSON for the same reason the answer is: the registry serves every domain and reads none of their argument types.
   * It is what lets a window that did not start a run still name what is running.
   *
   * Absent for a job whose command described nothing.
   */
  request: unknown | null;
  /**
   * Whether stopping has been asked for. A job can carry this and still be running: cancellation lands at a boundary
   * the operation chooses, and the gap between asking and stopping is exactly what a reader wants to see.
   */
  isCancelRequested: boolean;
  /**
   * The job's own progress: live for a running job, as last seen for a finished one.
   *
   * Absent for a job registered but not yet reporting — a run holding a lease while it validates its inputs, say.
   */
  progress: JobProgress | null;
  /** Absent while the job is running. */
  conclusion: JobConclusion | null;
  /** Why it failed, where it did. */
  error: string | null;
  /**
   * What the run answered, for a job that completed.
   *
   * JSON rather than a type, because the registry serves every domain and none of their result types are its
   * business. The tool that started the work is the one that knows how to read it.
   *
   * Absent while the job runs, and for a job that failed or was cancelled before it had an answer.
   */
  result: unknown | null;
  /** How long the job ran, measured by the registry rather than by the operation. */
  duration: number;
};

/** The application operations that can be registered, cancelled and rediscovered. */
export type JobKind =
  | "archives.extract"
  | "archives.compare"
  | "archives.pack"
  | "archives.patch"
  | "archives.unpack"
  | "configs.check-format"
  | "configs.format"
  | "configs.verify"
  | "spawn.pack"
  | "spawn.unpack"
  | "sprite-equipment.pack"
  | "gamedata.verify"
  | "textures.build"
  | "textures.compare-encodings"
  | "textures.make-bump"
  | "textures.save"
  | "translations.build"
  | "translations.check-format"
  | "translations.format"
  | "translations.parse"
  | "translations.verify";

/** What an equipment sprite pack was asked to do. */
export type PackSpriteRequest = {
  /** Directory of loose icons to draw from. */
  sourcePath: string;
  /** Sheet to write. */
  outputPath: string;
  /** `system.ltx` declaring which icons exist and where they sit. */
  systemLtxPath: string;
  /** Whether to resolve that config with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
};

/** What a path is right now, for a form field that has to say what it points at before a command runs. */
export type PathDescription = {
  kind: PathKind;
  /** Entries directly inside a directory. */
  entryCount: number | null;
};

/**
 * What a path turned out to be.
 *
 * Carried instead of a pair of booleans so the states cannot disagree. Failing to look is not a variant here: it
 * reaches the caller as an error, which is what lets a refused check read as unknown rather than as absent.
 */
export type PathKind = "missing" | "file" | "directory";

/**
 * What the viewer is showing, paired with where it came from.
 *
 * The enclosing snapshot supplies the geometry identity; source and roots describe its inputs and texture lookups.
 */
export type SelectedVisualDescription = {
  source: VisualSource;
  /** The roots used to resolve this selection's texture files. */
  roots: XrayRoots;
  description: VisualDescription;
  dependencies: VisualDependencies;
  /** What each located texture file is, keyed by the logical path that located it. */
  textures: { [key in string]: AssetTextureDescriptor };
  /** What the renderer builds for each declared texture, keyed by the reference as the mesh declares it. */
  materials: { [key in string]: XrayMaterialDescriptor };
  /** How the renderer draws each declared shader, keyed by the shader name as the mesh declares it. */
  surfaces: { [key in string]: XraySurfaceDescriptor };
  /** A `textures.ltx` the searched roots hold, or `None`. */
  texturesLtx: XrayAsset | null;
};

/** A single opening, allocated by its caller before dispatch so it can also be closed while pending. */
export type SessionId = string;

/**
 * The optional committed snapshot returned during restoration.
 *
 * A named wire type also keeps the generic parameter scoped when Specta exports nullable results.
 */
export type SessionRestore<T> = SessionSnapshot<T> | null;

/** An immutable value addressed by the opening that produced it; a held snapshot survives close. */
export type SessionSnapshot<T> = {
  sessionId: SessionId;
  value: T;
};

/** The conversion performed, retained with the result after a window reload. */
export type SpawnConversion = "pack" | "unpack";

/** Source and output paths of a standalone spawn conversion. */
export type SpawnConversionRequest = {
  source: string;
  destination: string;
};

/** What reached disk; cancellation is accepted only before writing begins. */
export type SpawnConversionResult = {
  operation: SpawnConversion;
  destination: string;
  outcome: JobOutcome;
};

/** One coherent opening, restored without reading the large chunks. */
export type SpawnSessionDescriptor = {
  sessionId: SessionId;
  path: string;
  header: SpawnHeaderChunk;
};

/** The complete identity and inputs of one sprite opening. */
export type SpriteEquipmentOpenRequest = {
  sessionId: SessionId;
  equipmentDdsPath: string;
  systemLtxPath: string;
  isDltx: boolean;
};

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

/** The roots and listing mode restored after a frontend reload. */
export type TextureBrowseSession = {
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
  /**
   * Whether the texture was written or the run stopped before it started.
   *
   * A cancelled build wrote nothing. There is one boundary and it is before the work: the encode writes the file
   * itself and is a single call with no seam inside it. That costs nothing worth having, because the encode is tens
   * of milliseconds - a descriptor decides the format here and `ETFormat` has no name for BC7, so the one candidate
   * that takes seconds cannot arise.
   */
  outcome: JobOutcome;
  destination: string;
  /** Size of the source, which the descriptor's own width and height are refreshed from. */
  width: number;
  height: number;
  /** Levels written, counting the base. */
  mipmapLevels: number;
  /** Recipe fields the descriptor asked for and the build did not carry out. */
  omissions: Array<TextureBuildOmissionReport>;
};

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
  /**
   * `.dds` files the roots hold outside `textures\`, which no engine reference names and this catalog leaves out.
   *
   * Counted rather than dropped silently: a level's lightmaps are the usual case, and a person wondering where a file
   * went deserves the number.
   */
  outsideTexturesCount: number;
};

/**
 * How a listing addressed what it found.
 *
 * Two shapes rather than one because the two cases want opposite defaults. In a game tree the files that yield no
 * engine reference are a level's lightmaps, and burying two thousand named textures in them is the bug; in a folder
 * somebody is authoring in, those files are the entire point and there are no references to be had at all.
 */
export type TextureCatalogMode =
  /** The game tree: listed by engine reference, archives included, files outside `textures\` counted not listed. */
  | "roots"
  /** A plain directory: every `.dds` under it, addressed by its own path. */
  | "looseDirectory";

/** Everything the inspection panels say about one texture, resolved in one call. */
export type TextureDescription = {
  source: TextureSource;
  /** What to call this texture on screen: its engine reference, or a standalone file's own stem. */
  reference: string;
  /** The roots the description was resolved in, so a later read searches what this searched. */
  roots: XrayRoots;
  /** The `.dds` the reference resolves to, or `None` for a descriptor with no texture. */
  texture: XrayAsset | null;
  /** What the base texture file is, when it is located and its bytes can be reached. */
  base: AssetTextureDescriptor | null;
  /**
   * What the renderer would build for this texture, or `None` for a file outside every tree.
   *
   * Absent rather than empty for a standalone file. There is no tree to resolve a bump pair or a detail against, so
   * answering "declares nothing" would be a claim this description is in no position to make - the descriptor beside
   * the file may well declare a pair, and what the engine would do with it depends on a game tree nobody has named.
   */
  material: XrayMaterialDescriptor | null;
  /** What the bound bump file is, when the material binds one and its bytes can be reached. */
  bump: AssetTextureDescriptor | null;
  /** What the bound bump companion file is, on the same terms. */
  companion: AssetTextureDescriptor | null;
  /** The descriptor's editable fields, when a `.thm` was located and parsed. */
  form: TextureDescriptorForm | null;
  /** Where an edit of this texture would write, absent for a texture served out of an archive. */
  targets: TextureEditTargets | null;
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
  /**
   * The whole `STextureParams` flag word, named bits and unnamed alike.
   *
   * One word rather than a boolean per bit: the twelve the SDK names are the ones a surface offers, and the rest are
   * bits somebody's tool set that this editor has no business dropping.
   */
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

/** The descriptor half of a save. */
export type TextureDescriptorSave = {
  target: TextureSaveTarget;
  form: TextureDescriptorForm;
};

/**
 * Where an edit of one texture would write, and what was there when the editor read it.
 *
 * Resolved by the command that located the files rather than derived by the frontend, for the same reason the
 * descriptor is: a path assembled in TypeScript out of a reference and a separator is a guess about where the VFS
 * found something, and the two disagree the moment a root is nested or a name is cased differently.
 *
 * Absent for a texture served out of an archive, which has no file to replace at all.
 */
export type TextureEditTargets = {
  /**
   * The `.thm` to write, whether or not one is there yet.
   *
   * Always present, because the editor can author a descriptor for a texture that has none: its
   * [`TextureSaveTarget::expected`] is what says which of the two cases this is.
   */
  descriptor: TextureSaveTarget;
  /** The `.dds` to replace when a re-encode is saved. */
  texture: TextureSaveTarget;
};

/** Every candidate weighed against one texture, with the texture itself for a baseline. */
export type TextureEncodingComparison = {
  sessionId: SessionId;
  source: TextureSource;
  roots: XrayRoots;
  /**
   * Whether every candidate was weighed or the run stopped because it was asked to.
   *
   * A cancelled comparison still reports what it managed, and the session still holds those encodes: a candidate it
   * reached is a real measurement and a real set of bytes, whatever happened after it.
   */
  outcome: JobOutcome;
  /** The texture the encodes were made from, which a save has to name to claim them. */
  reference: string;
  current: TextureEncodingCurrent;
  /** The candidates weighed, in the order [`DdsEncodeCandidate::ALL`] lists them, cheapest first. */
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

/**
 * A format the base texture can be written in, of the five worth offering.
 *
 * A plugin-side mirror of [`DdsEncodeCandidate`] rather than the crate's own enum, for the reason every wire type
 * here is one: `xrf-dds` is a pure image crate and carries no bindings feature, and a surface naming a format wants
 * a name that cannot change under it.
 */
export type TextureEncodingFormat = "bc1" | "bc2" | "bc3" | "rgba8" | "bc7";

/** How hard the encoder works, which trades seconds for fidelity. */
export type TextureEncodingQuality =
  | "fast"
  | "normal"
  /** The default, because everything but BC7 costs pennies at it. */
  | "slow";

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

/** The texture half of a save, which is one of the candidates a comparison already encoded. */
export type TextureEncodingSave = {
  sessionId: SessionId;
  target: TextureSaveTarget;
  format: TextureEncodingFormat;
};

/** One texture name and the files the roots hold for it. */
export type TextureEntry = {
  /**
   * What to call this row: an engine reference such as `ston\ston_beton05`, or a loose file's path below its root.
   *
   * Unique within one listing either way, so a tree can key on it. It is a label rather than an address; what to
   * open is `source`, because a loose file has no reference to be resolved back into.
   */
  reference: string;
  /** How to open this row, which is the address the describe and every write take. */
  source: TextureSource;
  role: TextureRole;
  /** The `.dds` the roots answer for this reference, winner first. */
  texture: XrayAsset | null;
  /** The `.thm` the roots answer for this reference, winner first. */
  descriptor: XrayAsset | null;
};

/**
 * What the editor read at a path, so a later write cannot overwrite a change it never saw.
 *
 * Size and modification time rather than a hash of the bytes: a texture is megabytes, the editor holds one node at a
 * time for minutes rather than days, and the case worth catching is an SDK or a converter having rewritten the file in
 * the meantime - which moves both.
 */
export type TextureFileStamp = {
  size: number;
  /** Milliseconds since the Unix epoch, as the platform reports the file's modification time. */
  modifiedMs: number;
};

/** One bit of the `STextureParams` flag word. */
export type TextureFlagEntry = {
  /** The bit, as a mask rather than an index, because the SDK's own bits are not contiguous. */
  bit: number;
  label: string;
};

/** What a generated pair came to. */
export type TextureMakeBumpOutcome = {
  /**
   * Whether the pair was written or the run stopped because it was asked to.
   *
   * A cancelled run wrote neither half: both are encoded before either is written, so there is no point at which
   * stopping could leave one half of a pair on disk with the other missing.
   */
  outcome: JobOutcome;
  /** The normals and gloss, written as `<name>_bump.dds`. */
  bump: string;
  /** The compression error and the height, written as `<name>_bump#.dds`. */
  companion: string;
  /** Mean gloss over the whole surface, in `0..=1`. */
  glossPower: number | null;
  /**
   * Whether the gloss is too dark for the surface to show a specular response worth having.
   *
   * A verdict rather than a failure, exactly as in the SDK: the pair is written either way, because a modder who
   * meant to author a matte surface is not making a mistake and one who did not wants to be told.
   */
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
  /** `supported`, `unsupported`, or `unverified` - the third being a path nobody has read, not a refusal. */
  support: string;
};

/**
 * What a texture name is by convention, before any descriptor has been read.
 *
 * Read off the name so a tree can fold a pair under its texture the moment the listing arrives; which pairs are
 * declared, and by whom, is what the sweep then says. The convention itself is `xrf-material`'s, shared with the
 * renderer's fallback rule and the companion derivation.
 */
export type TextureRole =
  /** A texture a mesh or a level binds by name. */
  | "texture"
  /** The first half of a bump pair: packed normal and gloss. */
  | "bump"
  /** The second half of a bump pair: packed error and height. */
  | "bumpCompanion";

/** What a save left on disk. */
export type TextureSaveOutcome = {
  /**
   * Whether the save wrote what it was asked to or stopped because it was asked to.
   *
   * A cancelled save wrote nothing. Cancellation is read once, after both files have been prepared and before either
   * is written, because there is no useful boundary inside two staged writes: stopping between them would leave a
   * texture whose descriptor still describes the old one, which is the state a save exists to avoid.
   */
  outcome: JobOutcome;
  /** The files written, in the order they were written. */
  written: Array<string>;
  /**
   * The format the descriptor ended up naming, when writing a texture changed it.
   *
   * Reported rather than left to the caller to infer, because the rule is the SDK's: `tfDXT1` and `tfADXT1` are one
   * encoder distinguished by the alpha flag, so only the descriptor's own flags can say which of them a BC1 texture
   * is. Absent when nothing synced - no texture was written, or its format has no name in `ETFormat`.
   */
  descriptorFormat: number | null;
};

/** One file a write addresses, and what was there when the editor read it. */
export type TextureSaveTarget = {
  /** Absolute path of the file to replace or create. */
  path: string;
  /** The stamp the editor read there, or `None` for a file it is creating. */
  expected: TextureFileStamp | null;
};

/** Where a texture is named from. */
export type TextureSource =
  /** A loose `.dds` or `.thm` on disk, named by its filesystem path. */
  | { kind: "file"; path: string }
  /** A texture of the roots, loose or archived, named by its engine reference such as `ston\ston_beton05`. */
  | { kind: "asset"; reference: string };

/**
 * Every named value the descriptor form's numeric fields can take.
 *
 * Answered once when the editor opens rather than carried on every description: it is the same table for every
 * texture in every root, and a description that repeated it would spend it thousands of times over a sweep.
 */
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
  /**
   * The bump mode that makes the engine bind a pair.
   *
   * Named rather than left to a surface to recognise, because a tool that has just written a pair has to point the
   * descriptor at it and there is exactly one value that does. Matching on the display label would work until
   * somebody rewords it; matching on the number would work until it is spelled differently in two places.
   */
  bumpModeUse: number;
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
  /** Path of the image to encode, of whatever kind `image` decodes. */
  source: string;
  /**
   * The descriptor to read as a recipe, as the editor currently has it rather than as it is on disk.
   *
   * The form rather than the file, because a build should produce what the editor's own format and flags describe. A
   * person who has changed the format and not saved yet wants to see that format built.
   */
  descriptor: TextureDescriptorForm;
  quality: TextureEncodingQuality;
};

/** What a format comparison was asked to weigh. */
export type TexturesCompareRequest = {
  sessionId: SessionId;
  /**
   * The texture to re-encode, named the way `describe` names one.
   *
   * A source rather than an engine reference, because a file outside every tree has no reference and is addressed by
   * its path. The label a surface shows still comes from the description; this is the address.
   */
  source: TextureSource;
  roots: XrayRoots;
  /**
   * Kernel the chain is reduced with, by its SDK name, or `None` to weigh the base level alone.
   *
   * Not read from the descriptor. A comparison answers "what would this texture cost in each format", and the answer
   * has to be about one chain built one way, or the figures are not comparable with each other.
   */
  mipFilter: string | null;
  quality: TextureEncodingQuality;
};

/**
 * What a bump pair generation was asked to do.
 *
 * The height source is required and everything else refines it, exactly as the SDK's generator has it: normals are
 * derived from the height alone and gloss is a separate plane, so a caller with only a height map still gets a pair.
 */
export type TexturesMakeBumpRequest = {
  /** Path of the texture the pair belongs to, without the `_bump` suffix or an extension. */
  destination: string;
  /** Path of the image the relief is read from, averaged across its colour channels. */
  height: string;
  /**
   * Path of a gloss mask, averaged the same way.
   *
   * When absent the whole surface takes [`Self::gloss_constant`], which is what a texture authored without a mask
   * needs and what the SDK's own dialog offers.
   */
  gloss: string | null;
  glossConstant: number | null;
  /** Path of a normal map to use instead of deriving one from the height, of the same size. */
  normalMap: string | null;
  /** `bump_virtual_height` of the descriptor, read here and nowhere at runtime. */
  virtualHeight: number | null;
  /**
   * Kernel the pair's chain is reduced with, by its SDK name.
   *
   * Defaults to `Box` at the caller, because that is what the SDK's generator leaves it at: `DXTCompressBump` builds
   * its `STextureParams` and overrides only the flags, the type and the format.
   */
  mipFilter: string | null;
  quality: TextureEncodingQuality;
};

/**
 * What one node's save was asked to write.
 *
 * Both halves are optional and independent: a node may be dirty in its descriptor, in its pixels, or in both, and a
 * save that could only do the pair would make the common case - a flag changed on a texture nobody re-encoded -
 * impossible to express.
 */
export type TexturesSaveRequest = {
  descriptor: TextureDescriptorSave | null;
  texture: TextureEncodingSave | null;
};

/**
 * What a build was asked to do.
 *
 * One argument rather than five, because a Tauri command's parameters are its wire signature and five of them plus a
 * job's own two is more than a reader can hold. It is also exactly what the registry retains, so a window adopting
 * this run after a reload sees the request rather than a summary of it.
 */
export type TranslationBuildRequest = {
  /** Where the sources are read from, through the VFS. */
  roots: XrayRoots;
  /** Where inside those roots to look, or nothing for the whole set. */
  prefix: string | null;
  /** The language to build, or `all`. */
  language: string;
  /** Directory the string tables are written into, which is always a host path. */
  outputDir: string;
  /** Whether to sort entries within each table. */
  isSorted: boolean;
};

/**
 * What a build reports back to the desktop surface.
 *
 * A row per language rather than the 272 files behind a full run, which is the natural grain of a
 * build whose job is one string table per language.
 */
export type TranslationBuildSummary = {
  /** Whether the run compiled every source or was stopped between them. */
  outcome: JobOutcome;
  /** The language built, or `all`. */
  language: string;
  /** Sources read. */
  sources: number;
  /** String tables written, across every language. */
  files: number;
  languages: Array<TranslationBuildLanguageSummary>;
};

/** One thing worth reporting about a file the run met. */
export type TranslationParseFinding = {
  rule: string;
  subject: string | null;
  message: string;
};

/**
 * What an import was asked to do.
 *
 * One argument rather than seven, because a Tauri command's parameters are its wire signature and seven of them plus
 * a job's own two is more than a reader can hold. It is also exactly what the registry retains, so a window adopting
 * this run after a reload sees the request rather than a summary of it.
 */
export type TranslationParseRequest = {
  /** Roots holding the raw XML, read through the VFS so an installation imports like a loose tree. */
  roots: XrayRoots;
  /** The language every entry this run reads is filed under. Never `all`. */
  language: string;
  /** Where inside those roots to look, or nothing to let the run resolve it. */
  prefix: string | null;
  /** Directory the JSON sources are written to, which may already hold some. */
  outputDir: string;
  /** Restrict the run to one table, by the file name it has in the scope. */
  file: string | null;
  /** Let incoming text replace existing text that differs, instead of keeping what is there. */
  isOverwrite: boolean;
  /** Do everything except write, so a caller can see what a run would change. */
  isDryRun: boolean;
};

/** What an import run reports back to the desktop surface. */
export type TranslationParseSummary = {
  /** Whether the run read every table or was stopped between them. */
  outcome: JobOutcome;
  /** The language every entry this run read was filed under. */
  language: string;
  /** Whether the run computed its answer without writing it. */
  isDryRun: boolean;
  census: TranslationParseCensus;
  findings: Array<TranslationParseFinding>;
};

/** How a save ended, once its edits were on disk. */
export type TranslationSaveOutcome =
  /** The edits are on disk, and this is the project as it now reads. */
  | { kind: "saved"; project: SessionSnapshot<TranslationProjectDescriptor> }
  /** The edits are on disk, but another project replaced this one while they were being written. */
  | { kind: "stale" };

/** What a completeness check reports back to the desktop surface. */
export type TranslationVerifySummary = {
  /**
   * Whether the run checked every source or was stopped between them.
   *
   * A stopped check reports the rows it reached; its silence about the rest is not a verdict.
   */
  outcome: JobOutcome;
  /** The language the check was narrowed to, or `all`. */
  language: string;
  /** Ids checked across every source. */
  checked: number;
  /** Ids with no text, counted once per language that lacks them. */
  missing: number;
  languages: Array<TranslationVerifyLanguageSummary>;
};

/** What a translation formatting run, or a check of one, was asked to do. */
export type TranslationsFormatRequest = {
  /** Project directory to format. */
  directory: string;
  /** Line endings to write, or nothing to keep what each file already uses. */
  lineEndings: string | null;
};

/** Inputs and caller-owned identity of one translations opening. */
export type TranslationsOpenRequest = {
  sessionId: SessionId;
  roots: XrayRoots;
  mode: TranslationProjectMode;
  prefix: string | null;
};

/** What a translation verification was asked to do. */
export type TranslationsVerifyRequest = {
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
  /** Language the check is about. */
  language: string;
};

/**
 * Where a visual is read from.
 *
 * Both variants are self-describing, and neither is a handle into mount state: an asset is named by its engine
 * identity, which any surface can spell without having opened anything. The roots it is looked for in travels beside
 * the source on every command that takes one, so one call can never mix two roots.
 */
export type VisualSource =
  /** A loose `.ogf` file on disk, named by its filesystem path. */
  | { kind: "file"; path: string }
  /** An asset of the roots, loose or archived, named by its engine identity. */
  | { kind: "asset"; logicalPath: string };

/** Backend job identities. */
export enum EJobKind {
  ARCHIVES_EXTRACT = "archives.extract",
  ARCHIVES_COMPARE = "archives.compare",
  ARCHIVES_PACK = "archives.pack",
  ARCHIVES_PATCH = "archives.patch",
  ARCHIVES_UNPACK = "archives.unpack",
  CONFIGS_CHECK_FORMAT = "configs.check-format",
  CONFIGS_FORMAT = "configs.format",
  CONFIGS_VERIFY = "configs.verify",
  SPAWN_PACK = "spawn.pack",
  SPAWN_UNPACK = "spawn.unpack",
  SPRITE_EQUIPMENT_PACK = "sprite-equipment.pack",
  GAMEDATA_VERIFY = "gamedata.verify",
  TEXTURES_BUILD = "textures.build",
  TEXTURES_COMPARE_ENCODINGS = "textures.compare-encodings",
  TEXTURES_MAKE_BUMP = "textures.make-bump",
  TEXTURES_SAVE = "textures.save",
  TRANSLATIONS_BUILD = "translations.build",
  TRANSLATIONS_CHECK_FORMAT = "translations.check-format",
  TRANSLATIONS_FORMAT = "translations.format",
  TRANSLATIONS_PARSE = "translations.parse",
  TRANSLATIONS_VERIFY = "translations.verify",
}
