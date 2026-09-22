import { Nullable } from "@xrf/types";

import { XrayAsset } from "@/core/ipc/types/xrf-vfs";
import { IEditorLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";

/**
 * Where an asset the VFS located was read out of.
 *
 * The archive rather than the name inside it, and the file rather than the entry: this fills a toolbar crumb, which
 * names the place a session is open and leaves what is open in it to the editor's own header.
 *
 * @param asset - The asset as the VFS reported it, or null when nothing is located.
 * @returns Where it is, or null when there is nothing to say.
 */
export function toAssetLocation(asset: Nullable<XrayAsset>): Nullable<IEditorLocation> {
  if (!asset) {
    return null;
  }

  if (asset.container.kind === "archive") {
    return { path: asset.container.path };
  }

  return { path: [asset.container.root, asset.container.relativePath].join(LOGICAL_PATH_SEPARATOR) };
}
