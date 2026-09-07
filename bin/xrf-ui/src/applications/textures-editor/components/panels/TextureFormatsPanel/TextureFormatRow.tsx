import { Box, Chip, ListItemButton, Typography } from "@mui/material";
import { ReactElement } from "react";

import { TextureEncodingReport } from "@/core/bindings/types/xrf-app";
import { EditorPanelRow } from "@/core/shell/editor/EditorPanel";
import { PANEL } from "@/core/theme/tokens";
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
      sx={{ alignItems: "stretch", borderRadius: 1, flexDirection: "column", gap: PANEL.sectionContentGap, py: 1 }}
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

      <Box>
        <EditorPanelRow label={"On disk"} value={formatBytes(candidate.fileBytes)} />
        <EditorPanelRow label={"Uploaded"} value={formatBytes(candidate.gpuBytes)} />
        <EditorPanelRow
          label={"Quality"}
          value={candidate.psnr === null ? "lossless" : `${candidate.psnr.toFixed(1)} dB additional`}
        />
        <EditorPanelRow label={"Encoding time"} value={`${candidate.encodeDuration} ms`} />
      </Box>
    </ListItemButton>
  );
}
