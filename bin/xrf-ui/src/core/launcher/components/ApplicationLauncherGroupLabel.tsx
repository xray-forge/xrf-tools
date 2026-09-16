import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { toAccentColor } from "@/core/launcher/lib";
import { IApplicationGroup } from "@/core/routing/application";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IApplicationLauncherGroupLabelProps extends BaseComponentProps {
  group: IApplicationGroup;
}

/**
 * The group a tool belongs to, wherever no section heading is there to say it.
 */
export function ApplicationLauncherGroupLabel({
  "data-testid": dataTestId = "application-launcher-group-label",
  id,
  className,
  group,
}: IApplicationLauncherGroupLabelProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-w-0 items-center gap-1.5", className)}>
      <span
        aria-hidden={true}
        className={"size-1.5 shrink-0 rounded-full"}
        style={{ backgroundColor: toAccentColor(group.accent) }}
      />

      <Typography
        className={"min-w-0 overflow-hidden text-ellipsis whitespace-nowrap text-text-secondary"}
        variant={"caption"}
      >
        {group.label}
      </Typography>
    </div>
  );
}
