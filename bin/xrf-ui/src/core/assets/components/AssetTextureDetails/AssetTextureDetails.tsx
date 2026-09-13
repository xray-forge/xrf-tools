import { ReactElement } from "react";

import { formatMipmapLevels } from "@/core/assets/lib/texture-shape";
import { AssetTextureDescriptor } from "@/core/ipc/types/xrf-app";
import { XrayAsset } from "@/core/ipc/types/xrf-vfs";
import { EditorPanelProperty } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

interface IAssetTextureDetailsProps extends BaseComponentProps {
  asset: XrayAsset;
  /** Absent when the located file could not be described; its size must not be reported as zero. */
  descriptor?: Nullable<AssetTextureDescriptor>;
}

/** Displays a texture file's location and header facts consistently in inspection panels. */
export function AssetTextureDetails({
  "data-testid": dataTestId,
  id,
  className,
  asset,
  descriptor,
}: IAssetTextureDetailsProps): ReactElement {
  const shape = descriptor?.shape;

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <EditorPanelProperty label={"Path"} value={asset.logicalPath} isMonospace />
      {asset.container.kind === "archive" ? (
        <EditorPanelProperty label={"Archive"} value={asset.container.path} isMonospace />
      ) : (
        <EditorPanelProperty label={"Root"} value={asset.container.root} isMonospace />
      )}
      {descriptor ? <EditorPanelProperty label={"Size"} value={formatBytes(descriptor.size)} /> : null}
      {shape ? (
        <>
          <EditorPanelProperty label={"Dimensions"} value={`${shape.width} × ${shape.height}`} />
          <EditorPanelProperty label={"Format"} value={shape.format} />
          <EditorPanelProperty label={"Mipmaps"} value={formatMipmapLevels(shape.mipmapLevels)} />
        </>
      ) : descriptor ? (
        <EditorPanelProperty label={"Format"} value={"Header unreadable"} />
      ) : null}
    </div>
  );
}
