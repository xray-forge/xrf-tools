import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { readPageScriptHeapDetail, TMemoryDetailSource, useMemoryUsageSegment } from "@/core/diagnostics/lib";
import { IEditorStatusSegment } from "@/core/shell/editor-shell";
import { ApplicationStatusSegment } from "@/core/shell/footer/ApplicationStatusSegment";

const SOURCES: ReadonlyArray<TMemoryDetailSource> = [readPageScriptHeapDetail];

/**
 * What the backend and the webview hold, at the end of every tool's status strip: the page's heap under the
 * processes in its hover.
 */
export function ApplicationMemoryStatus(): ReactElement {
  const segment: Nullable<IEditorStatusSegment> = useMemoryUsageSegment(SOURCES);

  // JSX is needed to wrap with `observer` automatically, `null` kills reactivity.
  return segment ? <ApplicationStatusSegment segment={segment} /> : <></>;
}
