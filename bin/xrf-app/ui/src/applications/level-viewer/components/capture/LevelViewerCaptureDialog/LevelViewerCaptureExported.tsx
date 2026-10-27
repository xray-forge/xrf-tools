import { Button, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { getPathName } from "@xrf/path";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback } from "react";

import { LevelViewerCaptureResultService } from "@/applications/level-viewer/services/capture-result";
import { cn } from "@/core/theme/class-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** Where the capture's last export was written, to show in its folder; nothing until one is written. */
export function LevelViewerCaptureExported({
  "data-testid": dataTestId = "level-viewer-capture-exported",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const resultService: LevelViewerCaptureResultService = useInjection(LevelViewerCaptureResultService);
  const exported: Nullable<string> = resultService.exported;

  const onReveal = useCallback(() => void resultService.revealExport(), [resultService]);

  if (!exported) {
    return <></>;
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
