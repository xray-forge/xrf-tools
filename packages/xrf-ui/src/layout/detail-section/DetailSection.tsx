import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { cn } from "#/theme";
import { Typography } from "#/typography";

export interface IDetailSectionProps extends BaseComponentProps {
  title: string;
  /** One sentence on what the section answers, and what it deliberately does not. Omit where the title says it. */
  description?: string;
  /** What the section currently amounts to, stated opposite the title: a total, a ratio, a count. */
  fact?: Nullable<string>;
  /** A control belonging to this section, such as which measurement its rows are ordered by. */
  action?: ReactNode;
  /** What the heading introduces. A section that only announces the rows beneath it has none. */
  children?: ReactNode;
}

/**
 * A titled block of detail: what it is, one sentence on why, whatever it currently amounts to, and its own control.
 */
export function DetailSection({
  "data-testid": dataTestId = "detail-section",
  className,
  id,
  title,
  description,
  fact = null,
  action,
  children,
}: IDetailSectionProps): ReactElement {
  const hasDetail: boolean = Boolean(description || action);

  return (
    <div data-testid={dataTestId} className={className} id={id}>
      <div className={cn("flex items-baseline justify-between gap-2", children && !hasDetail ? "mb-2" : null)}>
        <Typography className={"text-text-primary"} variant={"subtitle2"}>
          {title}
        </Typography>

        {fact ? (
          <Typography className={"shrink-0 text-text-secondary"} variant={"caption"}>
            {fact}
          </Typography>
        ) : null}
      </div>

      {hasDetail ? (
        <div className={cn("flex items-center justify-between gap-2", children ? "mb-2" : null)}>
          {description ? (
            <Typography className={"text-text-secondary"} variant={"caption"}>
              {description}
            </Typography>
          ) : null}

          {action ? <div className={"shrink-0"}>{action}</div> : null}
        </div>
      ) : null}

      {children}
    </div>
  );
}
