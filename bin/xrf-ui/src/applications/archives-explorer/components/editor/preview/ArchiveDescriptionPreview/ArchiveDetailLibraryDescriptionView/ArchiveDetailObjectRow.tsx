import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDetailEntry } from "@/core/ipc/types/xrf-app";
import { MONOSPACE, PANEL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionReferenceLink } from "../ArchiveDescriptionReferenceLink";
import { describeEntryPlanting } from "./ArchiveDetailLibraryDescriptionView.utils";
import { describeModelDetail } from "../ArchiveDetailModelSection/ArchiveDetailModelSection.utils";

interface IArchiveDetailObjectRowProps extends BaseComponentProps {
  entry: ArchiveDetailEntry;
}

/**
 * One object of a level's detail library: what it draws with, how widely it is planted, and what qualifies it.
 */
export function ArchiveDetailObjectRow({
  "data-testid": dataTestId = "archive-detail-object-row",
  id,
  className,
  entry,
}: IArchiveDetailObjectRowProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ minWidth: 0, paddingY: PANEL.propertyPaddingY, lineHeight: PANEL.contentLineHeight }}
    >
      <Box sx={{ display: "flex", gap: 1, justifyContent: "space-between", minWidth: 0 }}>
        <Typography variant={"body2"} sx={{ ...MONOSPACE, minWidth: 0, overflowWrap: "anywhere" }}>
          {entry.model.texture ? (
            <ArchiveDescriptionReferenceLink reference={entry.model.texture} />
          ) : (
            `Object ${entry.index}`
          )}
        </Typography>

        <Typography variant={"body2"} sx={{ color: "text.secondary", flexShrink: 0, whiteSpace: "nowrap" }}>
          {describeEntryPlanting(entry)}
        </Typography>
      </Box>

      <Typography variant={"caption"} sx={{ display: "block", color: "text.disabled", overflowWrap: "anywhere" }}>
        {describeModelDetail(entry.model)}
      </Typography>
    </Box>
  );
}
