// Auto-generated rust bindings. Do not edit it manually.

import { InventorySpriteDescriptor } from "@/core/bindings/types/xrf-texture";
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
 * Where a caller wants an asset looked for, named rather than handed over.
 *
 * Self-describing on purpose: a world is identified by what it is made of, never by a handle the backend issued. A
 * webview reload therefore loses nothing, and a surface that did not open a world can still address assets in it —
 * which is what lets one plugin's selection be read by another's preview.
 *
 * The subject asset belongs to the spec rather than to the command that has one, because every command taking this
 * world must search the same places: resolving a model's texture against the model's own tree and then reading it
 * without that tree is how a loose model came back with geometry and no textures.
 */
export type AssetWorldSpec = {
  /** Asset whose own X-Ray root and installation are searched first, when the world is centred on one. */
  asset: string | null;
  /** Roots searched after the asset's own, in the order given. */
  roots: Array<string>;
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
  /** The world the selection was opened in, so a reloaded frontend asks for geometry the same way. */
  world: AssetWorldSpec;
  description: VisualDescription;
  dependencies: VisualDependencies;
  /** What each located texture file is, keyed by the logical path that located it. */
  textures: { [key in string]: AssetTextureDescriptor };
};

/**
 * Where a visual is read from.
 *
 * Both variants are self-describing, and neither is a handle into mount state: an asset is named by its engine
 * identity, which any surface can spell without having opened anything. The world it is looked for in travels beside
 * the source on every command that takes one, so one call can never mix two worlds.
 */
export type VisualSource =
  /** A loose `.ogf` file on disk, named by its filesystem path. */
  | { kind: "file"; path: string }
  /** An asset of the world, loose or archived, named by its engine identity. */
  | { kind: "asset"; logicalPath: string };
