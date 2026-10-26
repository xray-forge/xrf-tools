import { Chip, Typography } from "@mui/material";
import { XrayAsset } from "@xrf/ipc/types/xrf-vfs";
import { VisualMotionDependency } from "@xrf/ipc/types/xrf-visual";
import { ReactElement } from "react";

import { listLocatedAssets } from "@/core/assets/lib/resolution";
import { cn } from "@/core/theme/class-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IVisualMotionRowProps extends BaseComponentProps {
  motion: VisualMotionDependency;
}

/**
 * One motion reference and whether it was found.
 */
export function VisualMotionRow({
  "data-testid": dataTestId = "visual-motion-row",
  id,
  className,
  motion,
}: IVisualMotionRowProps): ReactElement {
  const { resolution } = motion;
  const assets: Array<XrayAsset> = listLocatedAssets(resolution);

  return (
    <div data-testid={dataTestId} id={id} className={cn("py-0.75", className)}>
      <div className={"flex items-baseline justify-between gap-2"}>
        <Typography className={"min-w-0 break-all"} variant={"body2"}>
          {motion.reference}
        </Typography>

        {assets.length > 0 ? (
          <Chip
            className={"shrink-0"}
            size={"small"}
            color={"success"}
            variant={"outlined"}
            label={assets.length > 1 ? `${assets.length} files` : "Found"}
          />
        ) : (
          <Chip
            className={"shrink-0"}
            size={"small"}
            color={resolution.kind === "rejected" ? "error" : "warning"}
            variant={"outlined"}
            label={resolution.kind === "rejected" ? "Unusable" : "Not found"}
          />
        )}
      </div>

      {assets.map((asset: XrayAsset) => (
        <Typography key={asset.logicalPath} className={"block break-all text-text-secondary"} variant={"caption"}>
          {asset.logicalPath}
        </Typography>
      ))}
    </div>
  );
}
