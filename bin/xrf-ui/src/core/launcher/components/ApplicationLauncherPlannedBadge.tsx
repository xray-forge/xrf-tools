import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Marks a tool the roster names but does not implement yet.
 */
export function ApplicationLauncherPlannedBadge({
  "data-testid": dataTestId = "application-launcher-planned-badge",
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
