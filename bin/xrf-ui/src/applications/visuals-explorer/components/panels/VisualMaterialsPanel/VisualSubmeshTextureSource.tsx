import { ReactElement } from "react";

import { VisualPanelRow } from "@/applications/visuals-explorer/components/panels/VisualPanelRow";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IVisualSubmeshTextureSourceProps extends BaseComponentProps {
  asset: XrayAsset;
}

/**
 * Which file answered a texture reference, and where that file lives.
 */
export function VisualSubmeshTextureSource({ asset }: IVisualSubmeshTextureSourceProps): ReactElement {
  return (
    <>
      <VisualPanelRow label={"Path"} value={asset.logicalPath} />

      {asset.container.kind === "archive" ? (
        <VisualPanelRow label={"Archive"} value={asset.container.path} />
      ) : (
        <VisualPanelRow label={"Root"} value={asset.container.root} />
      )}
    </>
  );
}
