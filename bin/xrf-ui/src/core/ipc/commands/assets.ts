// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { XrayEngineResolution } from "@/core/ipc/types/xrf-engine-target";
import { EXrayAssetType, XrayAsset, XrayRootProbe, XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands */
export const assetsCommands = {
  /** Detect which engine roots target, as an open left to detection would read them. */
  detectEngine: (roots: XrayRoots) => __TAURI_INVOKE<XrayEngineResolution>("plugin:assets|detect_engine", { roots }),
  /** Every asset of one kind the roots hold, winner first and shadowed copies omitted. */
  listAssets: (roots: XrayRoots, kind: EXrayAssetType) =>
    __TAURI_INVOKE<Array<XrayAsset>>("plugin:assets|list_assets", { roots, kind }),
  /** Describe what a path is, without mounting it. */
  probeRoot: (path: string) => __TAURI_INVOKE<XrayRootProbe>("plugin:assets|probe_root", { path }),
};
