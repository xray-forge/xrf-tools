import { ReactElement } from "react";

import { IRendererLightsReport } from "@/core/render/lib/contract/renderer-lights-report";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatPercent } from "@/lib/format/number";

interface ILevelStreamLightsSectionProps extends BaseComponentProps {
  lights: IRendererLightsReport;
}

/**
 * What the local lights came to: how many stood in view, how many with their shadows, the atlas they are drawn in, and
 * what the frame could not hold.
 */
export function LevelStreamLightsSection({
  "data-testid": dataTestId = "level-stream-lights-section",
  id,
  className,
  lights,
}: ILevelStreamLightsSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Lights"}>
      <EditorPanelProperty label={"In view"} value={lights.inView} />
      <EditorPanelProperty label={"Shadowed"} value={lights.shadowed} />
      <EditorPanelProperty
        label={"Past the most a frame holds"}
        value={lights.excessLights ? `${lights.excessLights}, the farthest` : "none"}
      />
      <EditorPanelProperty
        label={"Shadow atlas"}
        value={
          `${formatPercent(lights.atlas.used / Math.max(lights.atlas.capacity, 1))} held · ` +
          `sizes at ${lights.shadowScale.toFixed(2)}×`
        }
      />
      <EditorPanelProperty
        label={"Full clusters"}
        value={lights.fullClusters ? `${lights.fullClusters} · ${lights.droppedLights} left out` : "none"}
      />
    </EditorPanelSection>
  );
}
