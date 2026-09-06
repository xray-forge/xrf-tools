import { Box, Chip, ListItemButton, Typography } from "@mui/material";
import { ReactElement } from "react";

import { TextureEncodingReport } from "@/core/bindings/types/xrf-app";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

interface ITextureFormatRowProps extends BaseComponentProps {
  candidate: TextureEncodingReport;
  isChosen: boolean;
  onChoose: () => void;
}

/**
 * One candidate format: what it would cost, what it would lose, and which renderers load it.
 */
export function TextureFormatRow({
  "data-testid": dataTestId = "texture-format-row",
  id,
  className,
  candidate,
  isChosen,
  onChoose,
}: ITextureFormatRowProps): ReactElement {
  return (
    <ListItemButton
      data-testid={dataTestId}
      id={id}
      className={className}
      selected={isChosen}
      onClick={onChoose}
      sx={{ alignItems: "flex-start", borderRadius: 1, flexDirection: "column", gap: 0.5, py: 1 }}
    >
      <Box sx={{ alignItems: "center", display: "flex", gap: 1, justifyContent: "space-between", width: "100%" }}>
        <Typography variant={"body2"}>{candidate.label}</Typography>

        <Chip
          size={"small"}
          variant={"outlined"}
          color={candidate.supportSummary.startsWith("all renderers") ? "default" : "warning"}
          label={candidate.supportSummary}
        />
      </Box>

      <Typography variant={"caption"} color={"text.secondary"}>
        {[
          formatBytes(candidate.fileBytes),
          `${formatBytes(candidate.gpuBytes)} uploaded`,
          candidate.psnr === null ? "lossless" : `${candidate.psnr.toFixed(1)} dB additional`,
          `${candidate.encodeDuration} ms`,
        ].join(" · ")}
      </Typography>
    </ListItemButton>
  );
}
