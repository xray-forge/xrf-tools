import { Chip } from "@mui/material";
import { ReactElement } from "react";

import { XraySurfaceDescriptor } from "@/core/bindings/types/xrf-material";
import {
  describeSurfaceDeclaration,
  describeSurfaceDraw,
  describeSurfaceOutcome,
  describeSurfaceShading,
  IMaterialStateDescriptor,
} from "@/core/materials/lib";
import { EditorPanelRow } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IVisualSubmeshSurfaceProps extends BaseComponentProps {
  /** What the backend resolved for this submesh's shader name, absent when it declares no shader. */
  surface: Nullable<XraySurfaceDescriptor>;
}

/**
 * Whether a submesh reads its texture's alpha channel, and what the renderer does with it.
 */
export function VisualSubmeshSurface({
  "data-testid": dataTestId,
  id,
  className,
  surface,
}: IVisualSubmeshSurfaceProps): Nullable<ReactElement> {
  if (!surface) {
    return null;
  }

  const outcome: IMaterialStateDescriptor = describeSurfaceOutcome(surface);
  const draw: Nullable<string> = describeSurfaceDraw(surface.draw);
  const shading: Nullable<string> = describeSurfaceShading(surface);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <EditorPanelRow
        label={"Alpha"}
        value={<Chip size={"small"} color={outcome.color} variant={"outlined"} label={outcome.label} />}
      />

      {draw ? <EditorPanelRow label={"Drawn"} value={draw} /> : null}

      <EditorPanelRow label={"Defined by"} value={describeSurfaceDeclaration(surface.declaration, surface.library)} />

      {shading ? <EditorPanelRow label={"Sorting"} value={shading} /> : null}
    </div>
  );
}
