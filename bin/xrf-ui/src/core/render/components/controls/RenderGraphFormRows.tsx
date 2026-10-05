import { ReactElement } from "react";

import { RenderGraphSettings } from "@/core/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IRenderGraphFormRowsProps extends BaseComponentProps {
  /** Which of the frame graph's mechanisms the frames compile with. */
  graph: RenderGraphSettings;
  onChange: (graph: RenderGraphSettings) => void;
}

/**
 * The frame graph's mechanisms, each to be turned off alone, or all for serial mode, to bisect a difference in a
 * capture; held for the session only.
 */
export function RenderGraphFormRows({
  "data-testid": dataTestId = "render-graph-form-rows",
  graph,
  onChange,
}: IRenderGraphFormRowsProps): ReactElement {
  return (
    <div data-testid={dataTestId}>
      <CheckboxFormRow
        label={"Cull unread passes"}
        description={"Drops passes whose output nothing reads."}
        isChecked={graph.isCulling}
        onChange={(isCulling: boolean) => onChange({ ...graph, isCulling })}
      />
      <CheckboxFormRow
        label={"Pool transients"}
        description={"Lets resources a frame makes for itself share memory when their lifetimes do not overlap."}
        isChecked={graph.isPooling}
        onChange={(isPooling: boolean) => onChange({ ...graph, isPooling })}
      />
      <CheckboxFormRow
        label={"Merge render passes"}
        description={"Draws consecutive passes into the same targets in one render pass."}
        isChecked={graph.isMerging}
        onChange={(isMerging: boolean) => onChange({ ...graph, isMerging })}
      />
      <CheckboxFormRow
        label={"Encode groups in parallel"}
        description={"Records the frame in a few encoders side by side; off, in one."}
        isChecked={graph.isGrouping}
        onChange={(isGrouping: boolean) => onChange({ ...graph, isGrouping })}
      />
    </div>
  );
}
