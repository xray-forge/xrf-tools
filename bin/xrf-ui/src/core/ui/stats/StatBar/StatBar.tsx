import { ReactElement } from "react";

import { cn } from "@/lib/dom/dom-name";
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
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("relative h-1.5 min-w-10 overflow-hidden rounded-surface bg-action-hover", className)}
      role={"img"}
      aria-label={label}
    >
      <div
        aria-hidden={true}
        className={"h-full bg-primary"}
        style={{ width: share > 0 ? `max(2px, ${(share * 100).toFixed(2)}%)` : 0 }}
      />
    </div>
  );
}
