import { ReactElement } from "react";

import { VisualPanelRow } from "@/applications/visuals-explorer/components/panels/VisualPanelRow";
import { AssetTextureDescriptor, AssetTextureShape } from "@/core/bindings/types/xrf-app";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

/**
 * States a mip chain in a way that distinguishes none from one.
 *
 * A file reporting a single level has no chain at all, which is why the loader has to drop it to a linear filter, and
 * saying "1 mip" would read as if it had one to spare.
 *
 * @param levels - Levels the header declares.
 * @returns A phrase describing the chain.
 */
function describeMipmaps(levels: number): string {
  return levels > 1 ? `${levels} mips` : "no mips";
}

/**
 * States a texture's layout in one line: its size in pixels, its format, and its mip chain.
 *
 * @param shape - Header facts the backend read.
 * @returns A single line describing the layout.
 */
function describeShape(shape: AssetTextureShape): string {
  return `${shape.width}×${shape.height} · ${shape.format} · ${describeMipmaps(shape.mipmapLevels)}`;
}

export interface IVisualSubmeshTextureSourceProps extends BaseComponentProps {
  asset: XrayAsset;
  /** What the file is, absent when it could not be reached to be described. */
  descriptor?: Nullable<AssetTextureDescriptor>;
}

/**
 * Which file answered a texture reference, where it lives, and what it is.
 *
 * The size is the file's own, which for a block-compressed texture is also what the renderer uploads. A located file
 * with no descriptor shows no size at all rather than a zero: unreachable is not empty.
 */
export function VisualSubmeshTextureSource({
  asset,
  descriptor = null,
}: IVisualSubmeshTextureSourceProps): ReactElement {
  return (
    <>
      <VisualPanelRow label={"Path"} value={asset.logicalPath} />

      {asset.container.kind === "archive" ? (
        <VisualPanelRow label={"Archive"} value={asset.container.path} />
      ) : (
        <VisualPanelRow label={"Root"} value={asset.container.root} />
      )}

      {descriptor ? <VisualPanelRow label={"Size"} value={formatBytes(descriptor.size)} /> : null}

      {descriptor?.shape ? <VisualPanelRow label={"Format"} value={describeShape(descriptor.shape)} /> : null}
    </>
  );
}
