import { Tooltip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { TEditorStatusSegment, useEditorStatusSegments } from "@/core/shell/editor-shell";
import { ApplicationStatusDetails } from "@/core/shell/footer/ApplicationStatusDetails";
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
  const segments: ReadonlyArray<TEditorStatusSegment> = useEditorStatusSegments();

  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex h-status-bar min-h-status-bar items-center justify-end gap-3 px-10", className)}
    >
      {/* Keyed by position or id, never by text: a key that follows a reading remounts its open tooltip. */}
      {segments.map((segment: TEditorStatusSegment, index: number) =>
        typeof segment === "string" ? (
          <Typography key={`text:${index}`} className={"text-text-secondary"} variant={"caption"} noWrap>
            {segment}
          </Typography>
        ) : (
          <Tooltip
            key={`segment:${segment.id}`}
            describeChild
            title={<ApplicationStatusDetails details={segment.details} />}
          >
            <Typography className={"text-text-secondary"} variant={"caption"} tabIndex={0} noWrap>
              {segment.text}
            </Typography>
          </Tooltip>
        )
      )}
    </div>
  );
}
