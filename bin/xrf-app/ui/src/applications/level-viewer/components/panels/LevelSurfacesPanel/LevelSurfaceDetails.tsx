import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip } from "@xrf/ui/material";
import { ReactElement } from "react";

import { describeLevelSurfacePass } from "@/applications/level-viewer/lib/surface/level-surface-pass";
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

interface ILevelSurfaceDetailsProps extends BaseComponentProps {
  summary: ILevelSurfaceSummary;
  /** What the renderer got for each texture the entry names, which is not always what the entry asked for. */
  dressing: ReadonlyArray<ILevelSurfaceDressing>;
  /** How much the entry draws across the resident sectors, which is the shape its shader cannot say. */
  geometry: ILevelSurfaceGeometry;
  /** Why the renderer has not said how much the entry draws, in place of the count; null once it has. */
  unmeasured?: Nullable<string>;
}

/**
 * The shader table entry chosen in the Surfaces panel: what it names, what it is drawn as, and what answered for it.
 */
export function LevelSurfaceDetails({
  "data-testid": dataTestId = "level-surface-details",
  id,
  className,
  summary,
  dressing,
  geometry,
  unmeasured = null,
}: ILevelSurfaceDetailsProps): ReactElement {
  const { descriptor } = summary;
  const outcome: IMaterialStateDescriptor = describeSurfaceOutcome(descriptor);
  const draw: Nullable<string> = describeSurfaceDraw(descriptor.draw);

  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={`${summary.shaderId} · ${describeLevelSurface(summary)}`}
      isFirst={true}
    >
      <EditorPanelProperty
        label={"Drawn as"}
        value={<Chip size={"small"} color={outcome.color} variant={"outlined"} label={outcome.label} />}
      />

      {draw ? <EditorPanelProperty label={"Which is"} value={draw} /> : null}

      <EditorPanelProperty label={"Drawn in"} value={describeLevelSurfacePass(descriptor)} />

      {dressing.length ? (
        <EditorPanelProperty
          label={"Dresses with"}
          value={dressing.map((it: ILevelSurfaceDressing) => (
            <div key={it.reference}>{describeLevelSurfaceDressing(it)}</div>
          ))}
        />
      ) : null}

      <EditorPanelProperty label={"Draws"} value={unmeasured ?? describeLevelSurfaceGeometry(geometry)} />

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
