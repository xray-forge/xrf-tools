import { Box } from "@mui/material";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IStatBarProps extends BaseComponentProps {
  /** What this row holds, against {@link total}. */
  value: number;
  /** What every row holds together. A zero total draws an empty track rather than dividing by it. */
  total: number;
  /** Named for a reader who cannot see the bar, since the figure beside it is already in the row. */
  label?: string;
}

/**
 * One row's share of a total, drawn as a proportion of a track.
 */
export function StatBar({
  "data-testid": dataTestId = "stat-bar",
  id,
  className,
  value,
  total,
  label,
}: IStatBarProps): ReactElement {
  const share: number = total > 0 ? Math.min(1, Math.max(0, value / total)) : 0;

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      role={"img"}
      aria-label={label}
      sx={{
        position: "relative",
        height: 6,
        minWidth: 40,
        borderRadius: 1,
        overflow: "hidden",
        backgroundColor: "action.hover",
      }}
    >
      <Box
        aria-hidden={true}
        sx={{
          // A hairline for anything that rounds to nothing: a row present in the table but invisible in the bar reads
          // as a rendering fault rather than as a small share.
          width: share > 0 ? `max(2px, ${(share * 100).toFixed(2)}%)` : 0,
          height: "100%",
          backgroundColor: "primary.main",
        }}
      />
    </Box>
  );
}
