import { Link, Typography } from "@mui/material";
import { ReactElement } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface ISettingsAboutRowProps extends BaseComponentProps {
  label: string;
  value: string;
  /** Where the value can be followed to, for the few that address something outside the application. */
  href?: Nullable<string>;
}

/**
 * One stated fact about this build or the machine running it.
 */
export function SettingsAboutRow({
  "data-testid": dataTestId = "settings-about-row",
  className,
  id,
  label,
  value,
  href = null,
}: ISettingsAboutRowProps): ReactElement {
  return (
    <div data-testid={dataTestId} className={cn("flex gap-2", className)} id={id}>
      <Typography className={"min-w-24 opacity-70"} variant={"caption"}>
        {label}
      </Typography>

      {href ? (
        <Link className={"monospace break-all"} href={href} target={"_blank"} rel={"noreferrer"} variant={"caption"}>
          {value}
        </Link>
      ) : (
        <Typography className={"monospace break-all"} variant={"caption"}>
          {value}
        </Typography>
      )}
    </div>
  );
}
