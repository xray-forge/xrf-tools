import { ReactElement } from "react";

import { formatMipmapLevels } from "@/core/assets/lib/texture-shape";
import { AssetTextureDescriptor } from "@/core/bindings/types/xrf-app";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { EditorPanelRow } from "@/core/shell/editor/EditorPanel";
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
      <EditorPanelRow label={"Path"} value={asset.logicalPath} isMonospace />
      {asset.container.kind === "archive" ? (
        <EditorPanelRow label={"Archive"} value={asset.container.path} isMonospace />
      ) : (
        <EditorPanelRow label={"Root"} value={asset.container.root} isMonospace />
      )}
      {descriptor ? <EditorPanelRow label={"Size"} value={formatBytes(descriptor.size)} /> : null}
      {shape ? (
        <>
          <EditorPanelRow label={"Dimensions"} value={`${shape.width} × ${shape.height}`} />
          <EditorPanelRow label={"Format"} value={shape.format} />
          <EditorPanelRow label={"Mipmaps"} value={formatMipmapLevels(shape.mipmapLevels)} />
        </>
      ) : descriptor ? (
        <EditorPanelRow label={"Format"} value={"Header unreadable"} />
      ) : null}
    </div>
  );
}
