import { Box, Chip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { VisualMotionDependency } from "@/core/bindings/types/xrf-visual";
import { listLocatedAssets } from "@/core/visuals/lib/visual-texture";
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
    <Box data-testid={dataTestId} id={id} className={className} sx={{ paddingY: 0.4 }}>
      <Box sx={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", gap: 1 }}>
        <Typography variant={"body2"} sx={{ minWidth: 0, wordBreak: "break-all" }}>
          {motion.reference}
        </Typography>

        {assets.length > 0 ? (
          <Chip
            size={"small"}
            color={"success"}
            variant={"outlined"}
            label={assets.length > 1 ? `${assets.length} files` : "Found"}
            sx={{ flexShrink: 0 }}
          />
        ) : (
          <Chip
            size={"small"}
            color={resolution.kind === "rejected" ? "error" : "warning"}
            variant={"outlined"}
            label={resolution.kind === "rejected" ? "Unusable" : "Not found"}
            sx={{ flexShrink: 0 }}
          />
        )}
      </Box>

      {assets.map((asset: XrayAsset) => (
        <Typography
          key={asset.logicalPath}
          variant={"caption"}
          sx={{ display: "block", color: "text.secondary", wordBreak: "break-all" }}
        >
          {asset.logicalPath}
        </Typography>
      ))}
    </Box>
  );
}
