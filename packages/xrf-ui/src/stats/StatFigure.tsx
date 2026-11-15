import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { BaseComponentProps } from "#/element-types";
import { cn } from "#/theme";
import { Typography } from "#/typography";

export interface IStatFigureProps extends BaseComponentProps {
  label: string;
  value: string;
  /** What the figure is of, where the label cannot carry it: a unit, a denominator, a count behind a total. */
  hint?: Nullable<string>;
}

/** One figure of a summary, named. */
export function StatFigure({
  "data-testid": dataTestId = "stat-figure",
  className,
  id,
  label,
  value,
  hint = null,
}: IStatFigureProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("min-w-26", className)}>
      <Typography className={"block text-text-secondary"} variant={"caption"}>
        {label}
      </Typography>

      <Typography variant={"body2"}>{value}</Typography>

      {hint ? (
        <Typography className={"block text-text-secondary opacity-70"} variant={"caption"}>
          {hint}
        </Typography>
      ) : null}
    </div>
  );
}
