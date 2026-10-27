import { AssetSource, EAssetSource } from "@xrf/ipc/types/xrf-app-core";
import { XrayAsset } from "@xrf/ipc/types/xrf-vfs";
import { Nullable } from "@xrf/types";
import { IEditorLocation } from "@xrf/ui/shell/editor/EditorToolbarLocation";

import { toAssetLocation } from "@/core/assets/lib";

/**
 * Where a single opened visual was read from, for a session that browses no root.
 *
 * The file, or the archive holding it - never the name inside that archive, which is what the preview's own header
 * says. A browsed session names its root instead and never reaches here.
 *
 * @param source - The open visual's source.
 * @param assets - Assets in the browsed roots, including their actual disk locations.
 * @returns The disk path, or null while the asset is not listed.
 */
export function toVisualLocation(
  source: Nullable<AssetSource>,
  assets: ReadonlyArray<XrayAsset>
): Nullable<IEditorLocation> {
  if (!source) {
    return null;
  }

  if (source.kind === EAssetSource.FILE) {
    return { path: source.path };
  }

  return toAssetLocation(assets.find((asset) => asset.logicalPath === source.logicalPath) ?? null);
}

/**
 * Where the visuals session is, which is what the toolbar crumb names.
 *
 * @param rootsLabel - Every root being browsed as one line, or null when nothing is browsed.
 * @param source - The open visual's source.
 * @param assets - Assets in the browsed roots, including their actual disk locations.
 * @returns What to put on the crumb, or null when there is nothing to say.
 */
export function toVisualSessionLocation(
  rootsLabel: Nullable<string>,
  source: Nullable<AssetSource>,
  assets: ReadonlyArray<XrayAsset>
): Nullable<IEditorLocation> {
  return rootsLabel ? { path: rootsLabel } : toVisualLocation(source, assets);
}
