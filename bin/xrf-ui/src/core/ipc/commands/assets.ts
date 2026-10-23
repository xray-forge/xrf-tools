// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { AssetChangeEvent } from "@/core/ipc/types/xrf-app";
import { XrayEngineResolution } from "@/core/ipc/types/xrf-engine-target";
import { EXrayAssetType, XrayAsset, XrayRevision, XrayRootProbe, XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands */
export const assetsCommands = {
  /**
   * Commit one text file as roots read it, the way an editor's save commits, answering the project revision it left: a
   * development build's way for a driver to change a file no editor writes yet. A release build refuses.
   */
  commitText: (roots: XrayRoots, logicalPath: string, text: string) =>
    __TAURI_INVOKE<XrayRevision>("plugin:assets|commit_text", { roots, logicalPath, text }),
  /** Detect which engine roots target, as an open left to detection would read them. */
  detectEngine: (roots: XrayRoots) => __TAURI_INVOKE<XrayEngineResolution>("plugin:assets|detect_engine", { roots }),
  /** Every asset of one kind the roots hold, winner first and shadowed copies omitted. */
  listAssets: (roots: XrayRoots, kind: EXrayAssetType) =>
    __TAURI_INVOKE<Array<XrayAsset>>("plugin:assets|list_assets", { roots, kind }),
  /** Describe what a path is, without mounting it. */
  probeRoot: (path: string) => __TAURI_INVOKE<XrayRootProbe>("plugin:assets|probe_root", { path }),
  /** Be told what every commit from now on changes: the project revision it leaves, and what it did to each open session. */
  subscribeChanges: (changes: Channel<AssetChangeEvent>) =>
    __TAURI_INVOKE<void>("plugin:assets|subscribe_changes", { changes }),
};
