import { XrayAsset } from "@xrf/ipc/types/xrf-vfs";
import { Nullable } from "@xrf/types";
import { IEditorLocation } from "@xrf/ui/editor/toolbar";

import { describeAssetContainer } from "@/core/assets/lib/container";

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

  return { path: describeAssetContainer(asset.container) };
}
