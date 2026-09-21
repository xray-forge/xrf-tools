import { Chip } from "@mui/material";
import { ReactElement } from "react";

import { describeLevelSurface, ILevelSurfaceSummary } from "@/core/level/lib/surface/level-surface-summary";
import {
  describeSurfaceDeclaration,
  describeSurfaceDraw,
  describeSurfaceOutcome,
  IMaterialStateDescriptor,
} from "@/core/materials/lib";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface ILevelSurfaceRowProps extends BaseComponentProps {
  summary: ILevelSurfaceSummary;
  isFirst?: boolean;
}

/**
 * One entry of a level's shader table: what it names, what it is drawn as, and what answered for it.
 */
export function LevelSurfaceRow({
  "data-testid": dataTestId = "level-surface-row",
  id,
  className,
  summary,
  isFirst = false,
}: ILevelSurfaceRowProps): ReactElement {
  const { descriptor } = summary;
  const outcome: IMaterialStateDescriptor = describeSurfaceOutcome(descriptor);
  const draw: Nullable<string> = describeSurfaceDraw(descriptor.draw);

  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={`${summary.shaderId} · ${describeLevelSurface(summary)}`}
      isFirst={isFirst}
    >
      <EditorPanelProperty
        label={"Drawn as"}
        value={<Chip size={"small"} color={outcome.color} variant={"outlined"} label={outcome.label} />}
      />

      {draw ? <EditorPanelProperty label={"Which is"} value={draw} /> : null}

      {summary.textures.length ? (
        <EditorPanelProperty label={"Dresses with"} value={summary.textures.join(", ")} />
      ) : null}

      <EditorPanelProperty
        label={"Read from"}
        value={describeSurfaceDeclaration(descriptor.declaration, descriptor.library)}
      />

      {descriptor.detail ? (
        <EditorPanelProperty
          label={"Detail"}
          value={`${descriptor.detail.reference} × ${(descriptor.detail.scale ?? 1).toFixed(1)}`}
        />
      ) : null}
    </EditorPanelSection>
  );
}
