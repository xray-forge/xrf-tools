import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { RenderGraphReport } from "@/core/ipc/types/xrf-renderer";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

interface ILevelRendererGraphSectionProps extends BaseComponentProps {
  /** What the frame graph made of the latest described frame, null before the first. */
  graph: Nullable<RenderGraphReport>;
}

/**
 * What the frame graph made of the frame: the passes the viewport and its window ran and those culled, the encode groups
 * and render passes, and the resources the frame made for itself, apart and as pooled.
 */
export function LevelRendererGraphSection({
  "data-testid": dataTestId = "level-renderer-graph-section",
  id,
  className,
  graph,
}: ILevelRendererGraphSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Frame graph"}>
      <EditorPanelProperty label={"Passes"} value={graph?.passes.length ?? 0} />
      <EditorPanelProperty label={"Culled"} value={graph?.culled.length ?? 0} />
      <EditorPanelProperty label={"Encode groups"} value={graph?.groups.length ?? 0} />
      <EditorPanelProperty label={"Render passes"} value={graph?.renderPassCount ?? 0} />
      <EditorPanelProperty label={"Transients"} value={graph?.transients.length ?? 0} />
      <EditorPanelProperty label={"Transients apart"} value={formatBytes(graph?.transientBytes ?? 0)} />
      <EditorPanelProperty
        label={"Pooled"}
        value={`${graph?.pooledCount ?? 0}, ${formatBytes(graph?.pooledBytes ?? 0)}`}
      />
    </EditorPanelSection>
  );
}
