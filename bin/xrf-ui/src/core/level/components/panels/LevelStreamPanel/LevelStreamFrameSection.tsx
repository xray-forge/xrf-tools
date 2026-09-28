import { ReactElement } from "react";

import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatMilliseconds } from "@/lib/format/duration";
import { formatCount } from "@/lib/format/number";

interface ILevelStreamFrameSectionProps extends BaseComponentProps {
  stats: ILevelStats;
}

/**
 * What a frame costs: its time and its worst, the drawing inside it, and what it drew.
 */
export function LevelStreamFrameSection({
  "data-testid": dataTestId = "level-stream-frame-section",
  id,
  className,
  stats,
}: ILevelStreamFrameSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Frame"}>
      <EditorPanelProperty label={"Frame time"} value={formatMilliseconds(stats.frameTime)} />
      <EditorPanelProperty label={"Worst frame"} value={formatMilliseconds(stats.worstFrameTime)} />
      <EditorPanelProperty
        label={"Drawing"}
        value={`${formatMilliseconds(stats.drawTime)} · worst ${formatMilliseconds(stats.worstDrawTime)}`}
      />
      <EditorPanelProperty label={"Frames a second"} value={stats.framesPerSecond.toFixed(0)} />
      <EditorPanelProperty label={"Taking a sector in"} value={formatMilliseconds(stats.sceneTime)} />
      <EditorPanelProperty label={"Draw calls"} value={formatCount(stats.draws)} />
      <EditorPanelProperty label={"Triangles"} value={formatCount(stats.triangles)} />
    </EditorPanelSection>
  );
}
