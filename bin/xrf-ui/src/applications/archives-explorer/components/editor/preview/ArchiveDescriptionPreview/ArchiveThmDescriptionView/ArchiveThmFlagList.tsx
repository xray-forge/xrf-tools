import { Chip } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveThmFlag } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveThmFlagListProps extends BaseComponentProps {
  flags: Array<ArchiveThmFlag>;
}

/**
 * Every bit the SDK names, set or not, under the spelling an author of a `.thm` would recognise.
 */
export function ArchiveThmFlagList({
  "data-testid": dataTestId = "archive-thm-flag-list",
  id,
  className,
  flags,
}: IArchiveThmFlagListProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-wrap gap-1", className)}>
      {flags.map((flag: ArchiveThmFlag) => (
        <Chip
          key={flag.label}
          className={flag.isSet ? "opacity-100" : "opacity-60"}
          size={"small"}
          variant={flag.isSet ? "filled" : "outlined"}
          label={flag.label}
        />
      ))}
    </div>
  );
}
