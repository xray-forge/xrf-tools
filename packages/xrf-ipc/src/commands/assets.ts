// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { AssetChangeEvent } from "#/types/xrf-app-core";
import { XrayEngineResolution } from "#/types/xrf-engine-target";
import { EXrayAssetType, XrayAsset, XrayRevision, XrayRootProbe, XrayRoots } from "#/types/xrf-vfs";

/** The wire name of every command of the `assets` plugin. */
export enum EAssetsCommand {
  COMMIT_TEXT = "plugin:assets|commit_text",
  DETECT_ENGINE = "plugin:assets|detect_engine",
  LIST_ASSETS = "plugin:assets|list_assets",
  PROBE_ROOT = "plugin:assets|probe_root",
  SUBSCRIBE_CHANGES = "plugin:assets|subscribe_changes",
}

/** Commands */
export const assetsCommands = {
  /**
   * Commit one text file as roots read it, the way an editor's save commits, answering the project revision it left: a
   * development build's way for a driver to change a file no editor writes yet. A release build refuses.
   */
  commitText: (roots: XrayRoots, logicalPath: string, text: string) =>
    __TAURI_INVOKE<XrayRevision>(EAssetsCommand.COMMIT_TEXT, { roots, logicalPath, text }),
  /** Detect which engine roots target, as an open left to detection would read them. */
  detectEngine: (roots: XrayRoots) => __TAURI_INVOKE<XrayEngineResolution>(EAssetsCommand.DETECT_ENGINE, { roots }),
  /**
   * Every asset of one kind the roots hold, winner first and shadowed copies omitted; read on the pool, since a first
   * listing plans and mounts the roots.
   */
  listAssets: (roots: XrayRoots, kind: EXrayAssetType) =>
    __TAURI_INVOKE<Array<XrayAsset>>(EAssetsCommand.LIST_ASSETS, { roots, kind }),
  /** Describe what a path is, without mounting it; on the pool, since a probe scans the directory. */
  probeRoot: (path: string) => __TAURI_INVOKE<XrayRootProbe>(EAssetsCommand.PROBE_ROOT, { path }),
  /** Be told what every commit from now on changes: the project revision it leaves, and what it did to each open session. */
  subscribeChanges: (changes: Channel<AssetChangeEvent>) =>
    __TAURI_INVOKE<void>(EAssetsCommand.SUBSCRIBE_CHANGES, { changes }),
};

/** What each command of the `assets` plugin takes and resolves to, keyed by its wire name. */
export interface IAssetsCommandSignatures {
  [EAssetsCommand.COMMIT_TEXT]: IIpcCommandSignature<
    typeof assetsCommands.commitText,
    ["roots", "logicalPath", "text"]
  >;
  [EAssetsCommand.DETECT_ENGINE]: IIpcCommandSignature<typeof assetsCommands.detectEngine, ["roots"]>;
  [EAssetsCommand.LIST_ASSETS]: IIpcCommandSignature<typeof assetsCommands.listAssets, ["roots", "kind"]>;
  [EAssetsCommand.PROBE_ROOT]: IIpcCommandSignature<typeof assetsCommands.probeRoot, ["path"]>;
  [EAssetsCommand.SUBSCRIBE_CHANGES]: IIpcCommandSignature<typeof assetsCommands.subscribeChanges, ["changes"]>;
}
