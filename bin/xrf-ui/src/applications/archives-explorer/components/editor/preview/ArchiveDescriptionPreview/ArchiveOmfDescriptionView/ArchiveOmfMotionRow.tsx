import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveOmfMotion } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
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
    <div data-testid={dataTestId} id={id} className={cn("min-w-0 py-1.5 leading-panel", className)}>
      <div className={"flex min-w-0 justify-between gap-2"}>
        <Typography className={"monospace min-w-0 wrap-anywhere"} variant={"body2"}>
          {motion.name}
        </Typography>

        <Typography className={"shrink-0 whitespace-nowrap text-text-secondary"} variant={"body2"}>
          {describeMotionLength(motion)}
        </Typography>
      </div>

      <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
        {describeMotionDetail(motion).join(" · ")}
      </Typography>
    </div>
  );
}
