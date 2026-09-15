import { Box, Chip } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveThmFlag } from "@/core/ipc/types/xrf-app";
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
    <Box data-testid={dataTestId} id={id} className={className} sx={{ display: "flex", flexWrap: "wrap", gap: 0.5 }}>
      {flags.map((flag: ArchiveThmFlag) => (
        <Chip
          key={flag.label}
          size={"small"}
          variant={flag.isSet ? "filled" : "outlined"}
          label={flag.label}
          sx={{ opacity: flag.isSet ? 1 : 0.6 }}
        />
      ))}
    </Box>
  );
}
