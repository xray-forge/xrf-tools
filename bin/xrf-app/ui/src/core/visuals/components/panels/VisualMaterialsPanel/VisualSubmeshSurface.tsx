import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip } from "@xrf/ui/material";
import { ReactElement } from "react";

import {
  describeSurfaceDeclaration,
  describeSurfaceDraw,
  describeSurfaceOutcome,
  describeSurfaceShading,
  IMaterialStateDescriptor,
} from "@/core/materials/lib";

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
      <EditorPanelProperty
        label={"Alpha"}
        value={<Chip size={"small"} color={outcome.color} variant={"outlined"} label={outcome.label} />}
      />

      {draw ? <EditorPanelProperty label={"Drawn"} value={draw} /> : null}

      <EditorPanelProperty
        label={"Defined by"}
        value={describeSurfaceDeclaration(surface.declaration, surface.library)}
      />

      {shading ? <EditorPanelProperty label={"Sorting"} value={shading} /> : null}
    </div>
  );
}
