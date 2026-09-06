import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { IEditorLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { Nullable } from "@/lib/types/general";

/** How the engine spells a path inside a tree, which is also how a volume spells its entries. */
const LOGICAL_SEPARATOR: string = "\\";

/**
 * Where an asset the VFS located actually is.
 *
 * @param asset - The asset as the VFS reported it, or null when nothing is located.
 * @returns Where it is, or null when there is nothing to say.
 */
export function toAssetLocation(asset: Nullable<XrayAsset>): Nullable<IEditorLocation> {
  if (!asset) {
    return null;
  }

  if (asset.container.kind === "archive") {
    return { entry: asset.logicalPath, path: asset.container.path };
  }

  return { path: [asset.container.root, asset.container.relativePath].join(LOGICAL_SEPARATOR) };
}
