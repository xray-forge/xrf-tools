import { useInjection } from "@wirestate/react";
import { formatBytes } from "@xrf/format";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback, useMemo } from "react";

import {
  ILevelViewerFileToastOpening,
  LevelViewerFileToast,
} from "@/tools/level-viewer/components/toast/LevelViewerFileToast";
import { ILevelViewerScreenshot } from "@/tools/level-viewer/lib/screenshot/level-viewer-screenshot";
import { LevelViewerScreenshotService } from "@/tools/level-viewer/services/level-viewer-screenshot";

/**
 * The screenshot just taken: its thumbnail opening it whole, its folder, its path, and its deletion.
 */
export function LevelViewerScreenshotToast({
  "data-testid": dataTestId = "level-viewer-screenshot-toast",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const screenshotService: LevelViewerScreenshotService = useInjection(LevelViewerScreenshotService);
  const shot: Nullable<ILevelViewerScreenshot> = screenshotService.shot.value;

  const opening: ILevelViewerFileToastOpening = useMemo(
    () => ({ label: "Open the screenshot", onOpen: () => screenshotService.setShown(true) }),
    [screenshotService]
  );

  const onReveal = useCallback(() => void screenshotService.reveal(), [screenshotService]);

  const onCopyPath = useCallback(() => void screenshotService.copyPath(), [screenshotService]);

  const onDelete = useCallback(() => void screenshotService.remove(), [screenshotService]);

  return (
    <LevelViewerFileToast
      data-testid={dataTestId}
      id={id}
      className={className}
      file={shot?.path ?? null}
      title={"Screenshot saved"}
      name={shot?.name ?? ""}
      details={shot ? `${shot.width} × ${shot.height} · ${formatBytes(shot.size)}` : ""}
      preview={shot ? <img className={"block h-14 w-24 object-cover"} src={shot.url} alt={""} /> : null}
      opening={opening}
      onReveal={onReveal}
      onCopyPath={onCopyPath}
      onDelete={onDelete}
    />
  );
}
