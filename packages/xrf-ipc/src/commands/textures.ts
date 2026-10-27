// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { SessionId, SessionRestore, SessionSnapshot } from "#/types/xrf-app-core";
import {
  ETextureCatalogMode,
  TextureBuildOutcome,
  TextureCatalog,
  TextureDescription,
  TextureEncodingComparison,
  TextureMakeBumpOutcome,
  TextureMaterialSummary,
  TextureSaveOutcome,
  TexturesBuildRequest,
  TexturesCompareRequest,
  TexturesMakeBumpRequest,
  TextureSource,
  TexturesSaveRequest,
  TexturesShowRequest,
  TextureVocabulary,
} from "#/types/xrf-app-textures";
import { JobProgress } from "#/types/xrf-job";
import { RenderViewportId } from "#/types/xrf-renderer";
import { XrayRoots } from "#/types/xrf-vfs";

/** The wire name of every command of the `textures` plugin. */
export enum ETexturesCommand {
  BUILD_FROM_SOURCE = "plugin:textures|build_from_source",
  CLOSE_BROWSE = "plugin:textures|close_browse",
  CLOSE_COMPARISON = "plugin:textures|close_comparison",
  COMPARE_ENCODINGS = "plugin:textures|compare_encodings",
  DESCRIBE = "plugin:textures|describe",
  DESCRIBE_CATALOG = "plugin:textures|describe_catalog",
  GET_VOCABULARY = "plugin:textures|get_vocabulary",
  MAKE_BUMP = "plugin:textures|make_bump",
  OPEN_BROWSE = "plugin:textures|open_browse",
  RESTORE_BROWSE = "plugin:textures|restore_browse",
  SAVE = "plugin:textures|save",
  SHOW_TEXTURE = "plugin:textures|show_texture",
}

/** Commands */
export const texturesCommands = {
  /** Rebuild a texture from a source image, the way its descriptor says to. */
  buildFromSource: (request: TexturesBuildRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureBuildOutcome>(ETexturesCommand.BUILD_FROM_SOURCE, { request, jobId, progress }),
  /** Close the listings `session_ids` name, committed or opening. */
  closeBrowse: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>(ETexturesCommand.CLOSE_BROWSE, { sessionIds }),
  /** Close the comparisons `session_ids` name, held or still weighing, letting their encodes go. */
  closeComparison: (sessionIds: Array<SessionId>) =>
    __TAURI_INVOKE<void>(ETexturesCommand.CLOSE_COMPARISON, { sessionIds }),
  /** Weigh every candidate format against one texture, and keep the encodes with the texture they were made of. */
  compareEncodings: (request: TexturesCompareRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureEncodingComparison>(ETexturesCommand.COMPARE_ENCODINGS, { request, jobId, progress }),
  /** Describe one texture: its file, its descriptor as the engine reads it, and the pair the engine binds. */
  describe: (source: TextureSource, roots: XrayRoots) =>
    __TAURI_INVOKE<TextureDescription>(ETexturesCommand.DESCRIBE, { source, roots }),
  /** Read every descriptor the roots hold and say what each makes of its texture. */
  describeCatalog: (roots: XrayRoots) =>
    __TAURI_INVOKE<Array<TextureMaterialSummary>>(ETexturesCommand.DESCRIBE_CATALOG, { roots }),
  /** The names the SDK gives the numbers a descriptor stores. */
  getVocabulary: () => __TAURI_INVOKE<TextureVocabulary>(ETexturesCommand.GET_VOCABULARY),
  /** Generate the `_bump` and `_bump#` pair a bumped surface binds, from a height map. */
  makeBump: (request: TexturesMakeBumpRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureMakeBumpOutcome>(ETexturesCommand.MAKE_BUMP, { request, jobId, progress }),
  /** Open a root set and list every texture it holds, read fresh as an open a person makes is. */
  openBrowse: (sessionId: SessionId, roots: XrayRoots, mode: ETextureCatalogMode) =>
    __TAURI_INVOKE<SessionSnapshot<TextureCatalog>>(ETexturesCommand.OPEN_BROWSE, { sessionId, roots, mode }),
  /** The listing `tool` browses in the asking window, the newest it opened, or null when it holds none. */
  restoreBrowse: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<TextureCatalog>>(ETexturesCommand.RESTORE_BROWSE, { tool }),
  /**
   * Write one node's descriptor, and the base texture with it when a held candidate is chosen: written over the texture
   * that candidate's comparison was made of, whose descriptor the request has to name.
   */
  save: (request: TexturesSaveRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureSaveOutcome>(ETexturesCommand.SAVE, { request, jobId, progress }),
  /**
   * Draw a texture laid on a body in a viewport, its files read by the renderer; no request draws none. Answers the
   * scene drawn, which the viewport's load reports name, or none.
   */
  showTexture: (viewport: RenderViewportId, request: TexturesShowRequest | null) =>
    __TAURI_INVOKE<number | null>(ETexturesCommand.SHOW_TEXTURE, { viewport, request }),
};

/** What each command of the `textures` plugin takes and resolves to, keyed by its wire name. */
export interface ITexturesCommandSignatures {
  [ETexturesCommand.BUILD_FROM_SOURCE]: IIpcCommandSignature<
    typeof texturesCommands.buildFromSource,
    ["request", "jobId", "progress"]
  >;
  [ETexturesCommand.CLOSE_BROWSE]: IIpcCommandSignature<typeof texturesCommands.closeBrowse, ["sessionIds"]>;
  [ETexturesCommand.CLOSE_COMPARISON]: IIpcCommandSignature<typeof texturesCommands.closeComparison, ["sessionIds"]>;
  [ETexturesCommand.COMPARE_ENCODINGS]: IIpcCommandSignature<
    typeof texturesCommands.compareEncodings,
    ["request", "jobId", "progress"]
  >;
  [ETexturesCommand.DESCRIBE]: IIpcCommandSignature<typeof texturesCommands.describe, ["source", "roots"]>;
  [ETexturesCommand.DESCRIBE_CATALOG]: IIpcCommandSignature<typeof texturesCommands.describeCatalog, ["roots"]>;
  [ETexturesCommand.GET_VOCABULARY]: IIpcCommandSignature<typeof texturesCommands.getVocabulary, []>;
  [ETexturesCommand.MAKE_BUMP]: IIpcCommandSignature<
    typeof texturesCommands.makeBump,
    ["request", "jobId", "progress"]
  >;
  [ETexturesCommand.OPEN_BROWSE]: IIpcCommandSignature<
    typeof texturesCommands.openBrowse,
    ["sessionId", "roots", "mode"]
  >;
  [ETexturesCommand.RESTORE_BROWSE]: IIpcCommandSignature<typeof texturesCommands.restoreBrowse, ["tool"]>;
  [ETexturesCommand.SAVE]: IIpcCommandSignature<typeof texturesCommands.save, ["request", "jobId", "progress"]>;
  [ETexturesCommand.SHOW_TEXTURE]: IIpcCommandSignature<typeof texturesCommands.showTexture, ["viewport", "request"]>;
}
