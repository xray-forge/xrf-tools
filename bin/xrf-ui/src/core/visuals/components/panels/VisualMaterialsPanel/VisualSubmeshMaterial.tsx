import { Chip } from "@mui/material";
import { ReactElement } from "react";

import { XrayMaterialDescriptor } from "@/core/bindings/types/xrf-material";
import { VisualPanelRow } from "@/core/visuals/components/panels/VisualPanelRow";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import {
  describeBumpDeclaration,
  describeBumpInput,
  describeBumpOutcome,
  describeDetail,
  describeVirtualHeight,
} from "./VisualSubmeshMaterial.utils";
import { IVisualTextureStateDescriptor } from "./VisualSubmeshTexture.utils";

export interface IVisualSubmeshMaterialProps extends BaseComponentProps {
  /** What the backend resolved for this submesh's texture reference, absent when it declares no texture. */
  material: Nullable<XrayMaterialDescriptor>;
}

/**
 * What the renderer builds for a submesh's texture beyond the diffuse: the bump declaration, both inputs it binds,
 * and the detail association.
 */
export function VisualSubmeshMaterial({ material }: IVisualSubmeshMaterialProps): ReactElement | null {
  if (!material) {
    return null;
  }

  const outcome: IVisualTextureStateDescriptor = describeBumpOutcome(material.outcome);
  const declaration: Nullable<string> = describeBumpDeclaration(material.declaration, material.descriptor);

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
    </>
  );
}
