import { ArchiveThmFlag } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip } from "@xrf/ui/material/chip";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

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
