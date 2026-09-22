import { Chip } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { describeLevelSurfaceDressing, ILevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import {
  describeLevelSurfaceGeometry,
  describeLevelSurfaceSpan,
  ILevelSurfaceGeometry,
} from "@/core/level/lib/surface/level-surface-geometry";
import { describeLevelSurface, ILevelSurfaceSummary } from "@/core/level/lib/surface/level-surface-summary";
import {
  describeSurfaceDeclaration,
  describeSurfaceDraw,
  describeSurfaceOutcome,
  IMaterialStateDescriptor,
} from "@/core/materials/lib";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelSurfaceRowProps extends BaseComponentProps {
  summary: ILevelSurfaceSummary;
  /** What the renderer got for each texture the entry names, which is not always what the entry asked for. */
  dressing: ReadonlyArray<ILevelSurfaceDressing>;
  /** How much the entry draws across the resident sectors, which is the shape its shader cannot say. */
  geometry: ILevelSurfaceGeometry;
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
  dressing,
  geometry,
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

      {dressing.length ? (
        <EditorPanelProperty
          label={"Dresses with"}
          value={dressing.map((it: ILevelSurfaceDressing) => (
            <div key={it.reference}>{describeLevelSurfaceDressing(it)}</div>
          ))}
        />
      ) : null}

      <EditorPanelProperty label={"Draws"} value={describeLevelSurfaceGeometry(geometry)} />

      {geometry.drawables ? (
        <EditorPanelProperty label={"Over"} value={describeLevelSurfaceSpan(geometry.span)} />
      ) : null}

      {geometry.drawables ? (
        <EditorPanelProperty label={"Narrowest draw"} value={describeLevelSurfaceSpan(geometry.narrowest)} />
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
