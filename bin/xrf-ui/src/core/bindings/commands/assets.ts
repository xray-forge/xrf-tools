// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import { XrayAsset, XrayAssetType, XrayRootProbe, XrayRoots } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const assetsCommands = {
  /**
   * Every asset of one kind the roots hold, winner first and shadowed copies omitted.
   *
   * Flat rather than a tree, because a flat index is what a filter reads and what a tree is built from — the same
   * shape the archive explorer already builds its file tree out of. The kind is the caller's, so listing a new kind is
   * an argument rather than a command.
   *
   * Assets keep the roots's own logical paths, so an entry names the model a `visuals` open can then take verbatim.
   */
  listAssets: (roots: XrayRoots, kind: XrayAssetType) =>
    __TAURI_INVOKE<Array<XrayAsset>>("plugin:assets|list_assets", { roots, kind }),
  /**
   * Describe what a path is, without mounting it.
   *
   * Answers the question a path setting asks and planning alone cannot: [`xrf_vfs::XrayMountMode::Auto`] plans any
   * readable directory as a root, so a source repository and a game data tree plan identically. The probe carries the
   * evidence that separates them, so a surface can say a directory holds nothing an engine would load.
   *
   * Cannot fail — an unreadable or absent path is one of the answers rather than an error.
   */
  probeRoot: (path: string) => __TAURI_INVOKE<XrayRootProbe>("plugin:assets|probe_root", { path }),
};
