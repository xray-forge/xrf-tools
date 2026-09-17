import { Chip, ListItemButton, Typography } from "@mui/material";
import { ReactElement } from "react";

import { TextureEncodingReport } from "@/core/ipc/types/xrf-app";
import { EditorPanelProperty } from "@/core/shell/editor/EditorPanel";
import { cn } from "@/lib/dom/dom-name";
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
      className={cn("flex-col items-stretch gap-2 rounded-surface py-2", className)}
      selected={isChosen}
      onClick={onChoose}
    >
      <div className={"flex w-full items-center justify-between gap-2"}>
        <Typography variant={"body2"}>{candidate.label}</Typography>

        <Chip
          size={"small"}
          variant={"outlined"}
          color={candidate.supportSummary.startsWith("all renderers") ? "default" : "warning"}
          label={candidate.supportSummary}
        />
      </div>

      <div>
        <EditorPanelProperty label={"On disk"} value={formatBytes(candidate.fileBytes)} />
        <EditorPanelProperty label={"Uploaded"} value={formatBytes(candidate.gpuBytes)} />
        <EditorPanelProperty
          label={"Quality"}
          value={candidate.psnr === null ? "lossless" : `${candidate.psnr.toFixed(1)} dB additional`}
        />
        <EditorPanelProperty label={"Encoding time"} value={`${candidate.encodeDuration} ms`} />
      </div>
    </ListItemButton>
  );
}
