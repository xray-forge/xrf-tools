import { useInjection } from "@wirestate/react";
import { getPathName } from "@xrf/path";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Button } from "@xrf/ui/material/button";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement, useCallback } from "react";

import { LevelViewerCaptureResultService } from "@/tools/level-viewer/services/level-viewer-capture-result";

/** Where the capture's last export was written, to show in its folder; nothing until one is written. */
export function LevelViewerCaptureExported({
  "data-testid": dataTestId = "level-viewer-capture-exported",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const resultService: LevelViewerCaptureResultService = useInjection(LevelViewerCaptureResultService);
  const exported: Nullable<string> = resultService.exported.value;

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
