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
          size={"small"}
          variant={flag.isSet ? "filled" : "outlined"}
          label={flag.label}
          sx={{ opacity: flag.isSet ? 1 : 0.6 }}
        />
      ))}
    </div>
  );
}
