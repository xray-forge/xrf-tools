// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import { XrayAsset, XrayAssetType, XrayWorldSpec } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const assetsCommands = {
  /**
   * Every asset of one kind a world holds, winner first and shadowed copies omitted.
   *
   * Flat rather than a tree, because a flat index is what a filter reads and what a tree is built from — the same
   * shape the archive explorer already builds its file tree out of. The kind is the caller's, so listing a new kind is
   * an argument rather than a command.
   *
   * Assets keep the world's own logical paths, so an entry names the model a `visuals` open can then take verbatim.
   */
  listAssets: (world: XrayWorldSpec, kind: XrayAssetType) =>
    __TAURI_INVOKE<Array<XrayAsset>>("plugin:assets|list_assets", { world, kind }),
};
