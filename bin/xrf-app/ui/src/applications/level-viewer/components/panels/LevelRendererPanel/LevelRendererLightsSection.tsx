import { formatCount, formatPercent } from "@xrf/format";
import { RenderLightsReport } from "@xrf/ipc/types/xrf-renderer";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

interface ILevelRendererLightsSectionProps extends BaseComponentProps {
  lights: RenderLightsReport;
}

/**
 * What the local lights came to: how many stood in view, how many with their shadows, the atlas they are drawn in, and
 * what the frame could not hold.
 */
export function LevelRendererLightsSection({
  "data-testid": dataTestId = "level-renderer-lights-section",
  id,
  className,
  lights,
}: ILevelRendererLightsSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Lamps"}>
      <EditorPanelProperty label={"In view"} value={formatCount(lights.inView)} />
      <EditorPanelProperty label={"Shadowed"} value={formatCount(lights.shadowed)} />
      <EditorPanelProperty
        label={"Past the most a frame holds"}
        value={lights.excess ? `${formatCount(lights.excess)}, the farthest` : "none"}
      />
      <EditorPanelProperty
        label={"Shadow atlas"}
        value={`${formatPercent(lights.atlas.used / Math.max(lights.atlas.capacity, 1))} held`}
      />
      <EditorPanelProperty
        label={"Full clusters"}
        value={
          lights.fullClusters ? `${formatCount(lights.fullClusters)} · ${formatCount(lights.dropped)} left out` : "none"
        }
      />
    </EditorPanelSection>
  );
}
