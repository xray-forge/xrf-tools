import { CircularProgress, Typography } from "@mui/material";
import { ReactElement } from "react";

import { CenteredColumn } from "@/core/ui/layout/CenteredColumn";
import { DELAYED_REVEAL_SX } from "@/core/ui/layout/delayed-reveal";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IDelayedProgressProps extends BaseComponentProps {
  /** Describes the pending read both visibly and to assistive technology. */
  label?: string;
  isOnViewport?: boolean;
}

/** Named progress and status text that stay hidden during fast operations. */
export function DelayedProgress({
  "data-testid": dataTestId,
  id,
  className,
  label = "Loading…",
  isOnViewport = false,
}: IDelayedProgressProps): ReactElement {
  return (
    <CenteredColumn data-testid={dataTestId} id={id} className={cn("gap-2 p-6", className)} sx={DELAYED_REVEAL_SX}>
      <CircularProgress aria-label={label} className={cn(isOnViewport && "text-viewport-accent")} />

      <Typography
        className={cn("text-text-secondary", isOnViewport ? "text-viewport-text" : null)}
        role={"status"}
        variant={"body2"}
      >
        {label}
      </Typography>
    </CenteredColumn>
  );
}
