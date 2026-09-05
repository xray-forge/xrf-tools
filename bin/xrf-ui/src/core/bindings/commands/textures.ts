// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import {
  TextureCatalog,
  TextureDescription,
  TextureMaterialSummary,
  TextureSource,
} from "@/core/bindings/types/xrf-app";
import { XrayRoot, XrayRoots } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const texturesCommands = {
  /** Stop browsing textures. */
  close: () => __TAURI_INVOKE<null>("plugin:textures|close"),
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
   * Open a root set and list every texture it holds.
   *
   * Lists and returns in one call, and reads no descriptor: the catalog is a walk of the mounted index, so the tree is
   * on screen before the sweep that badges it has started. `describe_catalog` is that sweep, asked for separately so a
   * person browses while it runs rather than waiting on it.
   */
  open: (roots: XrayRoots) => __TAURI_INVOKE<TextureCatalog>("plugin:textures|open", { roots }),
};
