import { Nullable } from "@xrf/types";
import { ReactElement, useMemo } from "react";

import { readPageScriptHeapDetail, TMemoryDetailSource, useMemoryUsageSegment } from "@/core/diagnostics/lib";
import { IEditorStatusSegment, useEditorMemoryDetails } from "@/core/shell/editor-shell";
import { ApplicationStatusSegment } from "@/core/shell/footer/ApplicationStatusSegment";

/**
 * What the backend and the webview hold, at the end of every application's status strip: the page's heap, then
 * whatever the application adds, under the processes in its hover.
 */
export function ApplicationMemoryStatus(): ReactElement {
  const details: ReadonlyArray<TMemoryDetailSource> = useEditorMemoryDetails();
  const sources: ReadonlyArray<TMemoryDetailSource> = useMemo(() => [readPageScriptHeapDetail, ...details], [details]);
  const segment: Nullable<IEditorStatusSegment> = useMemoryUsageSegment(sources);

  // JSX is needed to wrap with `observer` automatically, `null` kills reactivity.
  return segment ? <ApplicationStatusSegment segment={segment} /> : <></>;
}
