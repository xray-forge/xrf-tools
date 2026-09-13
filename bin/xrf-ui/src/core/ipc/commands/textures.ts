// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  SessionId,
  SessionRestore,
  SessionSnapshot,
  TextureBrowseSession,
  TextureBuildOutcome,
  TextureCatalog,
  TextureCatalogMode,
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
  /**
   * Rebuild a texture from a source image, the way its descriptor says to.
   *
   * The descriptor decides everything - the layout, whether there is a mip chain and which kernel reduces it - and
   * nothing is taken from the file being replaced, so a rebuild is reproducible from the two inputs alone. A format the
   * build has no honest encoder for is refused rather than substituted; a field it merely does not implement yet comes
   * back in [`TextureBuildOutcome::omissions`].
   *
   * Holds the file it would write, so a save or a generation aimed at it is refused rather than allowed to race it, and
   * joins the encode group for the same reason a generation does.
   */
  buildFromSource: (request: TexturesBuildRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureBuildOutcome>("plugin:textures|build_from_source", { request, jobId, progress }),
  /** Close browse state and invalidate held or unfinished comparisons. */
  close: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:textures|close", { sessionIds }),
  /**
   * Weigh every candidate format against one texture, and keep the encodes.
   *
   * The source is decoded once and reduced once; the candidates are then encoded from those same levels, so the
   * comparison weighs formats rather than weighing one format against a differently built chain. Every figure is
   * relative to the current file as decoded, which for a texture already stored in a DXT family is itself lossy - so
   * what is reported is the loss a re-encode adds, never distance from an original.
   */
  compareEncodings: (request: TexturesCompareRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureEncodingComparison>("plugin:textures|compare_encodings", { request, jobId, progress }),
  /**
   * Describe one texture: its file, its descriptor as the engine reads it, and the pair the engine binds.
   *
   * Resolution happens once, for the texture and both halves, inside one probe, so the three files are looked for in
   * the same roots: a second probe could mount a source between the calls and answer differently. The roots are
   * centred on the file when the source is one, and the effective roots travel back so a later read searches alike.
   */
  describe: (source: TextureSource, roots: XrayRoots) =>
    __TAURI_INVOKE<TextureDescription>("plugin:textures|describe", { source, roots }),
  /** Read every descriptor the roots hold and say what each makes of its texture. */
  describeCatalog: (roots: XrayRoots) =>
    __TAURI_INVOKE<Array<TextureMaterialSummary>>("plugin:textures|describe_catalog", { roots }),
  /** The session the explorer was browsing, or null when nothing is open. */
  getSession: () => __TAURI_INVOKE<SessionRestore<TextureBrowseSession>>("plugin:textures|get_session"),
  /**
   * The names the SDK gives the numbers a descriptor stores.
   *
   * A static table, asked for once when the editor opens. It takes no roots and reads nothing off disk: what a `tfDXT5`
   * or a `flBinaryAlpha` is called is a property of the format, not of the tree being edited.
   */
  getVocabulary: () => __TAURI_INVOKE<TextureVocabulary>("plugin:textures|get_vocabulary"),
  /** Generate the `_bump` and `_bump#` pair a bumped surface binds, from a height map. */
  makeBump: (request: TexturesMakeBumpRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureMakeBumpOutcome>("plugin:textures|make_bump", { request, jobId, progress }),
  /**
   * Open a root set and list every texture it holds.
   *
   * Lists and returns in one call, and reads no descriptor: the catalog is a walk of the mounted index, so the tree is
   * on screen before the sweep that badges it has started. `describe_catalog` is that sweep, asked for separately so a
   * person browses while it runs rather than waiting on it.
   */
  open: (sessionId: SessionId, roots: XrayRoots, mode: TextureCatalogMode) =>
    __TAURI_INVOKE<SessionSnapshot<TextureCatalog>>("plugin:textures|open", { sessionId, roots, mode }),
  /** Write one node's pending files: its descriptor, its base texture, or both. */
  save: (request: TexturesSaveRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureSaveOutcome>("plugin:textures|save", { request, jobId, progress }),
};
