import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { useEditorStatusSegments } from "@/core/shell/editor-shell";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Bottom status strip.
 */
export function ApplicationStatusBar({
  "data-testid": dataTestId = "application-status-bar",
  id = "application-status-bar",
  className,
}: BaseComponentProps): ReactElement {
  const segments: ReadonlyArray<string> = useEditorStatusSegments();

  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex h-status-bar min-h-status-bar items-center justify-end gap-3 px-10", className)}
    >
      {segments.length
        ? segments.map((segment: string, index: number) => (
            <Typography key={segment + index} variant={"caption"} noWrap sx={{ color: "text.secondary" }}>
              {segment}
            </Typography>
          ))
        : null}
    </div>
  );
}
