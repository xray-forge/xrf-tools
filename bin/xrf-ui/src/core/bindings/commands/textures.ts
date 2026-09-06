// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import {
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
} from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import { XrayRoot, XrayRoots } from "@/core/bindings/types/xrf-vfs";

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
  /** Stop browsing textures. */
  close: () => __TAURI_INVOKE<null>("plugin:textures|close"),
  /**
   * Weigh every candidate format against one texture, and keep the encodes.
   *
   * The source is decoded once and reduced once; the candidates are then encoded from those same levels, so the
   * comparison weighs formats rather than weighing one format against a differently built chain. Every figure is
   * relative to the current file as decoded, which for a texture already stored in a DXT family is itself lossy - so
   * what is reported is the loss a re-encode adds, never distance from an original.
   *
   * The encodes are held rather than returned. A `save` naming one of them writes the bytes this call produced, and the
   * next comparison replaces the session whole: an encode belonging to a texture nobody is looking at any more is not
   * something a later save should be able to reach.
   *
   * Writes nothing, so it holds no file. It joins the encode group because it spends the pool the way a generation
   * does, and it is the one command here where cancelling buys something real: the candidates are weighed cheapest
   * first, so a stop lands within tens of milliseconds unless BC7 has already begun.
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
  /** The roots the explorer was browsing, or null when nothing is open. */
  getRoots: () =>
    __TAURI_INVOKE<{
      /**
       * Native asset address whose own X-Ray root and installation are searched first, when the read is centred on one.
       *
       * This is what finds a texture shipped beside a model rather than in the shared tree.
       */
      asset: string | null;
      /** Roots searched after the asset's own, in the order given. */
      roots: Array<XrayRoot>;
    } | null>("plugin:textures|get_roots"),
  /**
   * Generate the `_bump` and `_bump#` pair a bumped surface binds, from a height map.
   *
   * Holds both halves it would write, so a save or a build aimed at either is refused rather than allowed to race it,
   * and joins the encode group: this is about two seconds of pool-saturating work on a large height map, and a second
   * one running beside it would not finish sooner for having started.
   *
   * The images are read here rather than on the blocking thread, before the job is registered, so a path that is not an
   * image is a request the caller got wrong rather than a run that started and failed.
   */
  makeBump: (request: TexturesMakeBumpRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureMakeBumpOutcome>("plugin:textures|make_bump", { request, jobId, progress }),
  /**
   * Open a root set and list every texture it holds.
   *
   * Lists and returns in one call, and reads no descriptor: the catalog is a walk of the mounted index, so the tree is
   * on screen before the sweep that badges it has started. `describe_catalog` is that sweep, asked for separately so a
   * person browses while it runs rather than waiting on it.
   */
  open: (roots: XrayRoots) => __TAURI_INVOKE<TextureCatalog>("plugin:textures|open", { roots }),
  /**
   * Write one node's pending files: its descriptor, its base texture, or both.
   *
   * Holds each file it would write, so a build or a generation aimed at the same texture is refused rather than allowed
   * to race it. It takes no action group: a save is two staged writes and refusing to save one node because another is
   * being compared would exclude nothing worth excluding.
   *
   * The encoded texture comes from the comparison this plugin is holding rather than from the request, because that is
   * where it already is - the webview was told what each candidate cost, not handed megabytes of block data to give
   * back. The bytes are taken under the session's lock and before the hop, so a re-selection arriving mid-save cannot
   * change what gets written.
   */
  save: (request: TexturesSaveRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TextureSaveOutcome>("plugin:textures|save", { request, jobId, progress }),
};
