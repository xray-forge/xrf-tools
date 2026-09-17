import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { toAccentColor } from "@/core/launcher/lib";
import { IApplicationGroup } from "@/core/routing/application";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IApplicationLauncherGroupHeadingProps extends BaseComponentProps {
  group: IApplicationGroup;
  count: number;
}

/**
 * The heading that opens one group's tools, in either view.
 */
export function ApplicationLauncherGroupHeading({
  "data-testid": dataTestId = "application-launcher-group-heading",
  id,
  className,
  group,
  count,
}: IApplicationLauncherGroupHeadingProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-w-0 items-center gap-1.5", className)}>
      <span aria-hidden={true} className={"flex [&>svg]:text-base"} style={{ color: toAccentColor(group.accent) }}>
        {group.icon}
      </span>

      <Typography component={"h2"} variant={"subtitle2"} className={"font-semibold text-text-primary"}>
        {group.label}
      </Typography>

      <Typography variant={"caption"} className={"text-text-secondary opacity-70"}>
        {count}
      </Typography>
    </div>
  );
}
