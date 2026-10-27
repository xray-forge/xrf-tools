import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement } from "react";

import { TEditorStatusSegment, useEditorStatusSegments } from "@/core/shell/editor-shell";
import { ApplicationMemoryStatus } from "@/core/shell/footer/ApplicationMemoryStatus";
import { ApplicationStatusSegment } from "@/core/shell/footer/ApplicationStatusSegment";

/**
 * Bottom status strip: what the application publishes, then what the process holds in memory.
 */
export function ApplicationStatusBar({
  "data-testid": dataTestId = "application-status-bar",
  id = "application-status-bar",
  className,
}: BaseComponentProps): ReactElement {
  const segments: ReadonlyArray<TEditorStatusSegment> = useEditorStatusSegments();

  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex h-status-bar min-h-status-bar items-center justify-end gap-3 px-10", className)}
    >
      {segments.map((segment: TEditorStatusSegment, index: number) => (
        <ApplicationStatusSegment
          key={typeof segment === "string" ? `text:${index}` : `segment:${segment.id}`}
          segment={segment}
        />
      ))}
      <ApplicationMemoryStatus />
    </div>
  );
}
