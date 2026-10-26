import { formatBytes } from "@xrf/format";
import { AssetTextureDescriptor } from "@xrf/ipc/types/xrf-app-core";
import { XrayAsset } from "@xrf/ipc/types/xrf-vfs";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { getAssetContainerSource, isLooseContainer } from "@/core/assets/lib/container";
import { formatMipmapLevels } from "@/core/assets/lib/texture-shape";
import { EditorPanelProperty } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
      <EditorPanelProperty
        label={isLooseContainer(asset.container) ? "Root" : "Archive"}
        value={getAssetContainerSource(asset.container)}
        isMonospace
      />
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
