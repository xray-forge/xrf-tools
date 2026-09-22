import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { formatLevelFacing, formatLevelPosition, ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { LevelViewportService } from "@/core/level/services";
import { RenderViewportOverlay } from "@/core/render/components/overlay";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
    <RenderViewportOverlay data-testid={dataTestId} id={id} className={className} corner={"bottom-right"}>
      <div>{formatLevelPosition(camera)}</div>
      <div>{formatLevelFacing(camera)}</div>
    </RenderViewportOverlay>
  );
}
