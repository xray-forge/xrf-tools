import { CircularProgress, Typography } from "@mui/material";
import { ReactElement } from "react";

import { CONTENT_STATE } from "@/core/theme/tokens";
import { CenteredColumn } from "@/core/ui/layout/CenteredColumn";
import { DELAYED_REVEAL_SX } from "@/core/ui/layout/delayed-reveal";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IDelayedProgressProps extends BaseComponentProps {
  /** Describes the pending read both visibly and to assistive technology. */
  label?: string;
}

/** Named progress and status text that stay hidden during fast operations. */
export function DelayedProgress({
  "data-testid": dataTestId,
  id,
  className,
  label = "Loading…",
}: IDelayedProgressProps): ReactElement {
  return (
    <CenteredColumn
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={[{ padding: CONTENT_STATE.padding, gap: CONTENT_STATE.gap }, DELAYED_REVEAL_SX]}
    >
      <CircularProgress aria-label={label} />

      <Typography role={"status"} variant={"body2"} sx={{ color: "text.secondary" }}>
        {label}
      </Typography>
    </CenteredColumn>
  );
}
