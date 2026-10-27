import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { readPageScriptHeapDetail, TMemoryDetailSource, useMemoryUsageSegment } from "@/core/diagnostics/lib";
import { ApplicationStatusSegment } from "@/core/shell/status-bar/ApplicationStatusSegment";
import { IShellStatusSegment } from "@/core/shell/status-bar/shell-status-segment";

const SOURCES: ReadonlyArray<TMemoryDetailSource> = [readPageScriptHeapDetail];

/**
 * What the backend and the webview hold, at the end of every tool's status strip: the page's heap under the
 * processes in its hover.
 */
export function ApplicationMemoryStatus(): ReactElement {
  const segment: Nullable<IShellStatusSegment> = useMemoryUsageSegment(SOURCES);

  // JSX is needed to wrap with `observer` automatically, `null` kills reactivity.
  return segment ? <ApplicationStatusSegment segment={segment} /> : <></>;
}
