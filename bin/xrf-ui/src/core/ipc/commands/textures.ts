// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  ETextureCatalogMode,
  SessionId,
  SessionRestore,
  SessionSnapshot,
  TextureBrowseSession,
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
  TextureVocabulary,
} from "@/core/ipc/types/xrf-app";
import { JobProgress } from "@/core/ipc/types/xrf-job";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands */
export const texturesCommands = {
  /** Rebuild a texture from a source image, the way its descriptor says to. */
  buildFromSource: (request: TexturesBuildRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureBuildOutcome>("plugin:textures|build_from_source", { request, jobId, progress }),
  /** Close browse state and invalidate held or unfinished comparisons. */
  close: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:textures|close", { sessionIds }),
  /** Weigh every candidate format against one texture, and keep the encodes. */
  compareEncodings: (request: TexturesCompareRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureEncodingComparison>("plugin:textures|compare_encodings", { request, jobId, progress }),
  /** Describe one texture: its file, its descriptor as the engine reads it, and the pair the engine binds. */
  describe: (source: TextureSource, roots: XrayRoots) =>
    __TAURI_INVOKE<TextureDescription>("plugin:textures|describe", { source, roots }),
  /** Read every descriptor the roots hold and say what each makes of its texture. */
  describeCatalog: (roots: XrayRoots) =>
    __TAURI_INVOKE<Array<TextureMaterialSummary>>("plugin:textures|describe_catalog", { roots }),
  /** The session the explorer was browsing, or null when nothing is open. */
  getSession: () => __TAURI_INVOKE<SessionRestore<TextureBrowseSession>>("plugin:textures|get_session"),
  /** The names the SDK gives the numbers a descriptor stores. */
  getVocabulary: () => __TAURI_INVOKE<TextureVocabulary>("plugin:textures|get_vocabulary"),
  /** Generate the `_bump` and `_bump#` pair a bumped surface binds, from a height map. */
  makeBump: (request: TexturesMakeBumpRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureMakeBumpOutcome>("plugin:textures|make_bump", { request, jobId, progress }),
  /** Open a root set and list every texture it holds. */
  open: (sessionId: SessionId, roots: XrayRoots, mode: ETextureCatalogMode) =>
    __TAURI_INVOKE<SessionSnapshot<TextureCatalog>>("plugin:textures|open", { sessionId, roots, mode }),
  /** Write one node's pending files: its descriptor, its base texture, or both. */
  save: (request: TexturesSaveRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureSaveOutcome>("plugin:textures|save", { request, jobId, progress }),
};
