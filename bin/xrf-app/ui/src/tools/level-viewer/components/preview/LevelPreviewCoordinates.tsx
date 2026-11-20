import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ViewportOverlay } from "@xrf/ui/media/viewport-overlay";
import { ReactElement } from "react";

import { formatLevelFacing, formatLevelPosition, ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { LevelViewportService } from "@/core/level/services/level-viewport";

/**
 * Where the camera stands, over the scene it is standing in.
 */
export function LevelPreviewCoordinates({
  "data-testid": dataTestId = "level-preview-coordinates",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewport: LevelViewportService = useInjection(LevelViewportService);
  const camera: Nullable<ILevelCamera> = viewport.camera;

  if (!camera) {
    return <></>;
  }

  return (
    <ViewportOverlay data-testid={dataTestId} id={id} className={className} corner={"bottom-right"}>
      <div>{formatLevelPosition(camera)}</div>
      <div>{formatLevelFacing(camera)}</div>
    </ViewportOverlay>
  );
}
