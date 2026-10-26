import { Button, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { getPathName } from "@xrf/path";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback } from "react";

import { LevelViewerCaptureService } from "@/applications/level-viewer/services/capture";
import { cn } from "@/core/theme/class-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** Where the capture's last export was written, to show in its folder; nothing until one is written. */
export function LevelViewerCaptureExported({
  "data-testid": dataTestId = "level-viewer-capture-exported",
  id,
  className,
}: BaseComponentProps): Nullable<ReactElement> {
  const captureService: LevelViewerCaptureService = useInjection(LevelViewerCaptureService);
  const exported: Nullable<string> = captureService.exported;

  const onReveal = useCallback(() => void captureService.revealExport(), [captureService]);

  if (!exported) {
    return null;
  }

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex items-center gap-2", className)}>
      <Typography className={"monospace truncate"} variant={"caption"} title={exported}>
        {`Exported ${getPathName(exported)}`}
      </Typography>
      <Button size={"small"} onClick={onReveal}>
        Show in folder
      </Button>
    </div>
  );
}
