import { formatBytes, formatCount } from "@xrf/format";
import { RenderGraphReport, RenderGraphSettings } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { RenderGraphFormRows } from "@/core/render/components/controls/RenderGraphFormRows";
import {
  DEFAULT_RENDER_GRAPH_SETTINGS,
  isRenderGraphBisected,
  SERIAL_RENDER_GRAPH_SETTINGS,
} from "@/core/render/lib/settings/render-graph-settings";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";

interface ILevelRendererGraphSectionProps extends BaseComponentProps {
  /** What the frame graph made of the latest described frame, null before the first. */
  graph: Nullable<RenderGraphReport>;
  /** Which of the frame graph's optimizations the frames compile with. */
  settings: RenderGraphSettings;
  onChange: (settings: RenderGraphSettings) => void;
}

/**
 * What the frame graph made of the frame: the passes the viewport and its window ran and those culled, the encode groups
 * and render passes, and the resources the frame made for itself, apart and as pooled; and its optimizations, each
 * turned off alone or all at once to bisect a difference in a capture.
 */
export function LevelRendererGraphSection({
  "data-testid": dataTestId = "level-renderer-graph-section",
  id,
  className,
  graph,
  settings,
  onChange,
}: ILevelRendererGraphSectionProps): ReactElement {
  const isBisected: boolean = isRenderGraphBisected(settings);

  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Frame graph"}>
      <EditorPanelProperty label={"Passes"} value={formatCount(graph?.passes.length ?? 0)} />
      <EditorPanelProperty label={"Culled"} value={formatCount(graph?.culled.length ?? 0)} />
      <EditorPanelProperty label={"Encode groups"} value={formatCount(graph?.groups.length ?? 0)} />
      <EditorPanelProperty label={"Render passes"} value={formatCount(graph?.renderPassCount ?? 0)} />
      <EditorPanelProperty label={"Transients"} value={formatCount(graph?.transients.length ?? 0)} />
      <EditorPanelProperty label={"Transients apart"} value={formatBytes(graph?.transientBytes ?? 0)} />
      <EditorPanelProperty
        label={"Pooled"}
        value={`${formatCount(graph?.pooledCount ?? 0)}, ${formatBytes(graph?.pooledBytes ?? 0)}`}
      />

      <CheckboxFormRow
        label={"Optimizations"}
        description={"Turn them off, alone or all at once, to bisect a difference in a capture."}
        isChecked={!isBisected}
        onChange={() => onChange(isBisected ? DEFAULT_RENDER_GRAPH_SETTINGS : SERIAL_RENDER_GRAPH_SETTINGS)}
      />

      <RenderGraphFormRows graph={settings} onChange={onChange} />
    </EditorPanelSection>
  );
}
