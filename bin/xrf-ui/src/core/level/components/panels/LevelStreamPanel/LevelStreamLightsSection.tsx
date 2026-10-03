import { ReactElement } from "react";

import { RenderLightsReport } from "@/core/ipc/types/xrf-renderer";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatPercent } from "@/lib/format/number";

interface ILevelStreamLightsSectionProps extends BaseComponentProps {
  lights: RenderLightsReport;
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
        value={lights.excess ? `${lights.excess}, the farthest` : "none"}
      />
      <EditorPanelProperty
        label={"Shadow atlas"}
        value={`${formatPercent(lights.atlas.used / Math.max(lights.atlas.capacity, 1))} held`}
      />
      <EditorPanelProperty
        label={"Full clusters"}
        value={lights.fullClusters ? `${lights.fullClusters} · ${lights.dropped} left out` : "none"}
      />
    </EditorPanelSection>
  );
}
