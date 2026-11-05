import { RenderGraphSettings } from "@xrf/ipc/types/xrf-renderer";
import { EditorPopoverSwitchRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

export interface IRenderGraphSwitchRowsProps extends BaseComponentProps {
  /** Which of the frame graph's mechanisms the frames compile with. */
  graph: RenderGraphSettings;
  /** Whether they sit in under a switch for them all. */
  isNested?: boolean;
  onChange: (graph: RenderGraphSettings) => void;
}

/**
 * The frame graph's mechanisms, a switch each, to be turned off alone, or all for serial mode, to bisect a difference
 * in a capture; held for the session only.
 */
export function RenderGraphSwitchRows({
  "data-testid": dataTestId = "render-graph-switch-rows",
  id,
  className,
  graph,
  isNested = false,
  onChange,
}: IRenderGraphSwitchRowsProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <EditorPopoverSwitchRow
        label={"Cull unread passes"}
        description={"Drops passes whose output nothing reads."}
        isChecked={graph.isCulling}
        isNested={isNested}
        onChange={(isCulling: boolean) => onChange({ ...graph, isCulling })}
      />
      <EditorPopoverSwitchRow
        label={"Pool transients"}
        description={"Lets resources a frame makes for itself share memory when their lifetimes do not overlap."}
        isChecked={graph.isPooling}
        isNested={isNested}
        onChange={(isPooling: boolean) => onChange({ ...graph, isPooling })}
      />
      <EditorPopoverSwitchRow
        label={"Merge render passes"}
        description={"Draws consecutive passes into the same targets in one render pass."}
        isChecked={graph.isMerging}
        isNested={isNested}
        onChange={(isMerging: boolean) => onChange({ ...graph, isMerging })}
      />
      <EditorPopoverSwitchRow
        label={"Encode groups in parallel"}
        description={"Records the frame in a few encoders side by side; off, in one."}
        isChecked={graph.isGrouping}
        isNested={isNested}
        onChange={(isGrouping: boolean) => onChange({ ...graph, isGrouping })}
      />
    </div>
  );
}
