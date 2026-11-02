import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { IShellContributed, useShellContributions } from "@/core/shell/contributions";
import { ApplicationMemoryStatus } from "@/core/shell/status-bar/ApplicationMemoryStatus";
import { ApplicationStatusSegment } from "@/core/shell/status-bar/ApplicationStatusSegment";
import { TShellStatusSegment } from "@/core/shell/status-bar/shell-status-segment";

/**
 * Bottom status strip: what the open tool publishes, then what the shell publishes, then what the process holds in
 * memory.
 */
export function ApplicationStatusBar({
  "data-testid": dataTestId = "application-status-bar",
  id = "application-status-bar",
  className,
}: BaseComponentProps): ReactElement {
  const contributed: IShellContributed<TShellStatusSegment> = useShellContributions("status");
  const segments: Array<TShellStatusSegment> = [...contributed.tool, ...contributed.shell];

  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex h-status-bar min-h-status-bar items-center justify-end gap-3 px-10", className)}
    >
      {segments.map((segment: TShellStatusSegment, index: number) => (
        <ApplicationStatusSegment
          key={typeof segment === "string" ? `text:${index}` : `segment:${segment.id}`}
          segment={segment}
        />
      ))}
      <ApplicationMemoryStatus />
    </div>
  );
}
