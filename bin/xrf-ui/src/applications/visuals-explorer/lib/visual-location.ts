import { toAssetLocation } from "@/core/assets/lib";
import { VisualSource } from "@/core/bindings/types/xrf-app";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { IEditorLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { Nullable } from "@/lib/types/general";

/**
 * Resolves the title-bar location from the file or the asset's reported container.
 *
 * @param source - The open visual's source.
 * @param assets - Assets in the browsed roots, including their actual disk locations.
 * @returns The disk path and optional archive entry, or null while the asset is not listed.
 */
export function toVisualLocation(
  source: Nullable<VisualSource>,
  assets: ReadonlyArray<XrayAsset>
): Nullable<IEditorLocation> {
  if (!source) {
    return null;
  }

  if (source.kind === "file") {
    return { path: source.path };
  }

  return toAssetLocation(assets.find((asset) => asset.logicalPath === source.logicalPath) ?? null);
}
