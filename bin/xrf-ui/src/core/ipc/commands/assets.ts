// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { EXrayAssetType, XrayAsset, XrayRootProbe, XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands */
export const assetsCommands = {
  /** Every asset of one kind the roots hold, winner first and shadowed copies omitted. */
  listAssets: (roots: XrayRoots, kind: EXrayAssetType) =>
    __TAURI_INVOKE<Array<XrayAsset>>("plugin:assets|list_assets", { roots, kind }),
  /** Describe what a path is, without mounting it. */
  probeRoot: (path: string) => __TAURI_INVOKE<XrayRootProbe>("plugin:assets|probe_root", { path }),
};
