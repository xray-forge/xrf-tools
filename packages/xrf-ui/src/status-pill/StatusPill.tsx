import { ReactElement } from "react";

import { BaseComponentProps } from "#/element-types";
import { cn } from "#/theme";
import { Typography } from "#/typography";

export interface IStatusPillProps extends BaseComponentProps {
  /** The status, in a word or two: "Planned", "Read only". */
  label: string;
}

/**
 * An outlined caption naming the state of what it sits beside, quieter than a chip and never interactive.
 */
export function StatusPill({
  "data-testid": dataTestId = "status-pill",
  id,
  className,
  label,
}: IStatusPillProps): ReactElement {
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
      {label}
    </Typography>
  );
}
