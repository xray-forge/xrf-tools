// Auto-generated rust bindings. Do not edit it manually.

import { JobOutcome, JobProgress } from "@/core/bindings/types/xrf-job";
import { InventorySpriteDescriptor } from "@/core/bindings/types/xrf-texture";
import {
  ProjectBuildLanguageSummary,
  ProjectParseCensus,
  ProjectVerifyLanguageSummary,
} from "@/core/bindings/types/xrf-translation";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { VisualDependencies, VisualDescription } from "@/core/bindings/types/xrf-visual";

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

export type EquipmentSpriteMetadata = {
  path: string;
  name: string;
  systemLtxPath: string;
  equipmentDescriptors: Array<InventorySpriteDescriptor>;
};

/**
 * One check's verdict, as the desktop surface shows it.
 *
 * The findings behind a check are deliberately not carried: a full run over an installation produces tens of
 * thousands of them, and a card that has to render all of them to say "meshes failed" is a card that never appears.
 * The count and the summary are what a person reads first; the detail belongs to a surface built to page through it.
 */
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
   * Whether every selected check ran, or the run was stopped between them.
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
  kind: string;
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

/**
 * What the viewer is showing, paired with where it came from.
 *
 * The source travels back so a frontend that reloaded knows what to ask geometry for, without having to remember
 * anything of its own across the reload.
 */
export type SelectedVisualDescription = {
  source: VisualSource;
  /** The roots the selection was opened in, so a reloaded frontend asks for geometry the same way. */
  roots: XrayRoots;
  description: VisualDescription;
  dependencies: VisualDependencies;
  /** What each located texture file is, keyed by the logical path that located it. */
  textures: { [key in string]: AssetTextureDescriptor };
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
  languages: Array<ProjectBuildLanguageSummary>;
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
  census: ProjectParseCensus;
  findings: Array<TranslationParseFinding>;
};

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
  languages: Array<ProjectVerifyLanguageSummary>;
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
