import { ReactElement } from "react";

import { RenderFrameReport } from "@/core/ipc/types/xrf-renderer";
import { describeRenderBackend } from "@/core/render/lib/settings/render-backend-choice";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatMilliseconds } from "@/lib/format/duration";

interface ILevelRendererFrameSectionProps extends BaseComponentProps {
  frame: RenderFrameReport;
}

/**
 * What a frame costs: its time and its worst, the render thread's own share, what it drew, and on what.
 */
export function LevelRendererFrameSection({
  "data-testid": dataTestId = "level-renderer-frame-section",
  id,
  className,
  frame,
}: ILevelRendererFrameSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Frame"}>
      <EditorPanelProperty label={"Frame time"} value={formatMilliseconds(frame.frameTime ?? 0)} />
      <EditorPanelProperty label={"Worst frame"} value={formatMilliseconds(frame.frameTimeMax ?? 0)} />
      <EditorPanelProperty label={"Render thread"} value={formatMilliseconds(frame.cpuTime ?? 0)} />
      <EditorPanelProperty label={"Frames a second"} value={(frame.framesPerSecond ?? 0).toFixed(0)} />
      <EditorPanelProperty label={"Taking a sector in"} value={formatMilliseconds(frame.sectorTime ?? 0)} />
      <EditorPanelProperty label={"Graphics API"} value={frame.adapter ? describeRenderBackend(frame.backend) : "—"} />
      <EditorPanelProperty label={"GPU"} value={frame.adapter || "—"} />
    </EditorPanelSection>
  );
}
