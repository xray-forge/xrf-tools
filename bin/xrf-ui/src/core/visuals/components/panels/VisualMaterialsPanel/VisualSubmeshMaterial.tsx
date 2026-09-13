import { Chip } from "@mui/material";
import { ReactElement } from "react";

import { XrayMaterialDescriptor } from "@/core/ipc/types/xrf-material";
import {
  describeBumpDeclaration,
  describeBumpInput,
  describeBumpOutcome,
  describeBumpShading,
  describeDetail,
  describeVirtualHeight,
  IMaterialStateDescriptor,
} from "@/core/materials/lib";
import { EditorPanelProperty } from "@/core/shell/editor/EditorPanel";
import { IVisualBumpStatus } from "@/core/visuals/lib/visual-bump";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { describeBumpUpload } from "./VisualSubmeshMaterial.utils";

interface IVisualSubmeshMaterialProps extends BaseComponentProps {
  /** What the backend resolved for this submesh's texture reference, absent when it declares no texture. */
  material: Nullable<XrayMaterialDescriptor>;
  /** What the frontend did with the bump pair, absent for a material that bound none. */
  status?: Nullable<IVisualBumpStatus>;
}

/**
 * What the renderer builds for a submesh's texture beyond the diffuse: the bump declaration, both inputs it binds,
 * the detail association, and what the viewer makes of them.
 */
export function VisualSubmeshMaterial({
  "data-testid": dataTestId,
  id,
  className,
  material,
  status = null,
}: IVisualSubmeshMaterialProps): ReactElement | null {
  if (!material) {
    return null;
  }

  const outcome: IMaterialStateDescriptor = describeBumpOutcome(material.outcome);
  const declaration: Nullable<string> = describeBumpDeclaration(material.declaration, material.descriptor);
  const shading: Nullable<string> = describeBumpShading(material);
  const upload: Nullable<string> = describeBumpUpload(status);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <EditorPanelProperty
        label={"Bump"}
        value={<Chip size={"small"} color={outcome.color} variant={"outlined"} label={outcome.label} />}
      />

      {declaration ? <EditorPanelProperty label={"Declared by"} value={declaration} /> : null}

      {material.bump ? (
        <>
          <EditorPanelProperty label={"Bump map"} value={describeBumpInput(material.bump.bump)} />
          <EditorPanelProperty label={"Bump#"} value={describeBumpInput(material.bump.companion)} />
          <EditorPanelProperty label={"Height"} value={describeVirtualHeight(material.bump.virtualHeight)} />
        </>
      ) : null}

      {material.detail ? <EditorPanelProperty label={"Detail"} value={describeDetail(material.detail)} /> : null}

      {shading ? <EditorPanelProperty label={"Shading"} value={shading} /> : null}

      {upload ? <EditorPanelProperty label={"Bump upload"} value={upload} /> : null}
    </div>
  );
}
