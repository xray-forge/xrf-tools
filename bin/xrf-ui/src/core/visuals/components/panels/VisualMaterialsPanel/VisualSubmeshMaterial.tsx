import { Chip } from "@mui/material";
import { ReactElement } from "react";

import { XrayMaterialDescriptor } from "@/core/bindings/types/xrf-material";
import { VisualPanelRow } from "@/core/visuals/components/panels/VisualPanelRow";
import { IVisualBumpStatus } from "@/core/visuals/lib/visual-bump";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import {
  describeBumpDeclaration,
  describeBumpInput,
  describeBumpOutcome,
  describeBumpShading,
  describeBumpUpload,
  describeDetail,
  describeVirtualHeight,
} from "./VisualSubmeshMaterial.utils";
import { IVisualTextureStateDescriptor } from "./VisualSubmeshTexture.utils";

export interface IVisualSubmeshMaterialProps extends BaseComponentProps {
  /** What the backend resolved for this submesh's texture reference, absent when it declares no texture. */
  material: Nullable<XrayMaterialDescriptor>;
  /** What the frontend did with the bump pair, absent for a material that bound none. */
  status?: Nullable<IVisualBumpStatus>;
}

/**
 * What the renderer builds for a submesh's texture beyond the diffuse: the bump declaration, both inputs it binds,
 * the detail association, and what the viewer makes of them.
 */
export function VisualSubmeshMaterial({ material, status = null }: IVisualSubmeshMaterialProps): ReactElement | null {
  if (!material) {
    return null;
  }

  const outcome: IVisualTextureStateDescriptor = describeBumpOutcome(material.outcome);
  const declaration: Nullable<string> = describeBumpDeclaration(material.declaration, material.descriptor);
  const shading: Nullable<string> = describeBumpShading(material);
  const upload: Nullable<string> = describeBumpUpload(status);

  return (
    <>
      <VisualPanelRow
        label={"Bump"}
        value={<Chip size={"small"} color={outcome.color} variant={"outlined"} label={outcome.label} />}
      />

      {declaration ? <VisualPanelRow label={"Declared by"} value={declaration} /> : null}

      {material.bump ? (
        <>
          <VisualPanelRow label={"Bump map"} value={describeBumpInput(material.bump.bump)} />
          <VisualPanelRow label={"Bump#"} value={describeBumpInput(material.bump.companion)} />
          <VisualPanelRow label={"Height"} value={describeVirtualHeight(material.bump.virtualHeight)} />
        </>
      ) : null}

      {material.detail ? <VisualPanelRow label={"Detail"} value={describeDetail(material.detail)} /> : null}

      {shading ? <VisualPanelRow label={"Shading"} value={shading} /> : null}

      {upload ? <VisualPanelRow label={"Bump upload"} value={upload} /> : null}
    </>
  );
}
