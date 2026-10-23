// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  ETextureCatalogMode,
  ETextureSurfaceAlpha,
  ETextureSurfaceShape,
  SessionId,
  SessionRestore,
  SessionScope,
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
import { RenderViewportId } from "@/core/ipc/types/xrf-renderer";
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
  /** The listing a tool was browsing, the newest it opened, or null when it holds none. */
  getSession: (scope: SessionScope) =>
    __TAURI_INVOKE<SessionRestore<TextureBrowseSession>>("plugin:textures|get_session", { scope }),
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
  /**
   * Draw a texture laid on a body in a viewport, its files read by the renderer; no request draws none. Answers the
   * scene drawn, which the viewport's load reports name, or none.
   */
  showTexture: (
    viewport: RenderViewportId,
    request: {
      source: TextureSource;
      /** The roots the texture is resolved in, as its description was. */
      roots: XrayRoots;
      shape: ETextureSurfaceShape;
      /** How many times the texture repeats across the body, which is how a tiling seam becomes visible. */
      tiling: number | null;
      alpha: ETextureSurfaceAlpha;
      /** Width over height of the texture, which the plane is stretched to. */
      aspect: number | null;
    } | null
  ) => __TAURI_INVOKE<number | null>("plugin:textures|show_texture", { viewport, request }),
};
