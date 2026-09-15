import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveOmfMotion } from "@/core/ipc/types/xrf-app";
import { MONOSPACE, PANEL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { describeMotionDetail, describeMotionLength } from "./ArchiveOmfDescriptionView.utils";

interface IArchiveOmfMotionRowProps extends BaseComponentProps {
  motion: ArchiveOmfMotion;
}

/**
 * One motion of a bank: its name, its length, and what qualifies it.
 */
export function ArchiveOmfMotionRow({
  "data-testid": dataTestId = "archive-omf-motion-row",
  id,
  className,
  motion,
}: IArchiveOmfMotionRowProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ minWidth: 0, paddingY: PANEL.propertyPaddingY, lineHeight: PANEL.contentLineHeight }}
    >
      <Box sx={{ display: "flex", gap: 1, justifyContent: "space-between", minWidth: 0 }}>
        <Typography variant={"body2"} sx={{ ...MONOSPACE, minWidth: 0, overflowWrap: "anywhere" }}>
          {motion.name}
        </Typography>

        <Typography variant={"body2"} sx={{ color: "text.secondary", flexShrink: 0, whiteSpace: "nowrap" }}>
          {describeMotionLength(motion)}
        </Typography>
      </Box>

      <Typography variant={"caption"} sx={{ display: "block", color: "text.disabled", overflowWrap: "anywhere" }}>
        {describeMotionDetail(motion).join(" · ")}
      </Typography>
    </Box>
  );
}
