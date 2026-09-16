import { Box } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveAnmDescription } from "@/core/ipc/types/xrf-app";
import { LAYOUT } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveAnmChannelsSection } from "./ArchiveAnmChannelsSection";
import { ArchiveAnmMotionSection } from "./ArchiveAnmMotionSection";

interface IArchiveAnmDescriptionViewProps extends BaseComponentProps {
  description: ArchiveAnmDescription;
}

/**
 * An object motion: how long the engine plays it, and what each of its channels does.
 */
export function ArchiveAnmDescriptionView({
  "data-testid": dataTestId = "archive-anm-description-view",
  id,
  className,
  description,
}: IArchiveAnmDescriptionViewProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ flexGrow: 1, minWidth: 0, minHeight: 0, overflowY: "auto" }}
    >
      <Box sx={{ maxWidth: LAYOUT.readingColumnWidth }}>
        <ArchiveAnmMotionSection description={description} />

        <ArchiveAnmChannelsSection channels={description.channels} />
      </Box>
    </Box>
  );
}
