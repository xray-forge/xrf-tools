import { Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IDetailSectionProps extends BaseComponentProps {
  title: string;
  /** One sentence on what the section answers, and what it deliberately does not. */
  description: string;
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
  return (
    <div data-testid={dataTestId} className={className} id={id}>
      <div className={"flex items-baseline justify-between gap-2"}>
        <Typography variant={"subtitle2"} sx={{ color: "text.primary" }}>
          {title}
        </Typography>

        {fact ? (
          <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0 }}>
            {fact}
          </Typography>
        ) : null}
      </div>

      <div
        className={
          children ? "mb-2 flex items-center justify-between gap-2" : "flex items-center justify-between gap-2"
        }
      >
        <Typography variant={"caption"} sx={{ color: "text.secondary" }}>
          {description}
        </Typography>

        {action ? <div className={"shrink-0"}>{action}</div> : null}
      </div>

      {children}
    </div>
  );
}
