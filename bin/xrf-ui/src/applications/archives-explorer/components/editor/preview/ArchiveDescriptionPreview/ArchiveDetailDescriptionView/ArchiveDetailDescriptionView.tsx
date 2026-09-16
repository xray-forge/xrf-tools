import { Box } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveDetailModel } from "@/core/ipc/types/xrf-app";
import { LAYOUT } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDetailModelSection } from "../ArchiveDetailModelSection";

interface IArchiveDetailDescriptionViewProps extends BaseComponentProps {
  description: ArchiveDetailModel;
  scope: ArchiveDescribeScope;
}

/**
 * A standalone detail object, which is one record and so one section.
 */
export function ArchiveDetailDescriptionView({
  "data-testid": dataTestId = "archive-detail-description-view",
  id,
  className,
  description,
  scope,
}: IArchiveDetailDescriptionViewProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ flexGrow: 1, minWidth: 0, minHeight: 0, overflowY: "auto" }}
    >
      <Box sx={{ maxWidth: LAYOUT.readingColumnWidth }}>
        <ArchiveDetailModelSection model={description} scope={scope} isFirst />
      </Box>
    </Box>
  );
}
