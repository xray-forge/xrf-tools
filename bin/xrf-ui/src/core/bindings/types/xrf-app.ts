// Auto-generated rust bindings. Do not edit it manually.

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
 * What a build reports back to the desktop surface.
 *
 * A row per language rather than the 272 files behind a full run, which is the natural grain of a
 * build whose job is one string table per language.
 */
export type TranslationBuildSummary = {
  /** The language built, or `all`. */
  language: string;
  /** Sources read. */
  sources: number;
  /** String tables written, across every language. */
  files: number;
  /** Where they were written, so the result can offer to open it. */
  outputDir: string;
  languages: Array<ProjectBuildLanguageSummary>;
};

/** One thing worth reporting about a file the run met. */
export type TranslationParseFinding = {
  rule: string;
  subject: string | null;
  message: string;
};

/** What an import run reports back to the desktop surface. */
export type TranslationParseSummary = {
  /** The language every entry this run read was filed under. */
  language: string;
  /** Whether the run computed its answer without writing it. */
  isDryRun: boolean;
  census: ProjectParseCensus;
  findings: Array<TranslationParseFinding>;
};

/** What a completeness check reports back to the desktop surface. */
export type TranslationVerifySummary = {
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
