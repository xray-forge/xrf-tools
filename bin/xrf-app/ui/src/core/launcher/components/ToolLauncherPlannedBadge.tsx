import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

/**
 * Marks a tool the roster names but does not implement yet.
 */
export function ToolLauncherPlannedBadge({
  "data-testid": dataTestId = "tool-launcher-planned-badge",
  id,
  className,
}: BaseComponentProps): ReactElement {
  return (
    <Typography
      data-testid={dataTestId}
      id={id}
      className={cn(
        "shrink-0 self-start rounded-surface border border-divider px-1.5",
        "text-badge leading-[1.7] font-semibold whitespace-nowrap text-text-secondary",
        className
      )}
      component={"span"}
      variant={"caption"}
    >
      Planned
    </Typography>
  );
}
