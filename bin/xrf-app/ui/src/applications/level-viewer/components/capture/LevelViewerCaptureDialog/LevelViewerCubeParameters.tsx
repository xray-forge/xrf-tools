import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ChoiceFormRow } from "@xrf/ui/form/choice-row";
import { Typography } from "@xrf/ui/material";
import { cn } from "@xrf/ui/theme";
import { ReactElement, useCallback } from "react";

import { LEVEL_CAPTURE_SETTLE_FRAMES } from "@/applications/level-viewer/lib/capture/level-capture";
import {
  ILevelCubeParameters,
  isLevelCubeSupersamplingDrawn,
  LEVEL_CUBE_SIZES,
  LEVEL_CUBE_SUPERSAMPLINGS,
  toLevelCubeCrossSize,
} from "@/applications/level-viewer/lib/cube/level-cube-capture";
import { LevelViewerCaptureService } from "@/applications/level-viewer/services/level-viewer-capture";

interface ILevelViewerCubeParametersProps extends BaseComponentProps {
  parameters: ILevelCubeParameters;
}

/**
 * What a cubemap is captured with: the size of its faces and how hard each is drawn. It is taken where the camera
 * stands, its faces along the world's axes, under the view's own look.
 */
export function LevelViewerCubeParameters({
  "data-testid": dataTestId = "level-viewer-cube-parameters",
  id,
  className,
  parameters,
}: ILevelViewerCubeParametersProps): ReactElement {
  const captureService: LevelViewerCaptureService = useInjection(LevelViewerCaptureService);
  const { size, supersampling, settleFrames } = parameters;
  const [crossWidth, crossHeight]: [number, number] = toLevelCubeCrossSize(size);

  const onSize = useCallback(
    (value: string) => captureService.setCubeParameters({ size: Number(value) }),
    [captureService]
  );

  const onSupersampling = useCallback(
    (value: string) => captureService.setCubeParameters({ supersampling: Number(value) }),
    [captureService]
  );

  const onSettleFrames = useCallback(
    (value: string) => captureService.setCubeParameters({ settleFrames: Number(value) }),
    [captureService]
  );

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-col gap-4", className)}>
      <Typography className={"text-text-secondary"} variant={"body2"}>
        Six faces around where the camera stands, along the world&apos;s axes in the engine&apos;s order, each exactly
        90 degrees square, under the view&apos;s weather, time and look; the clock is held while they are drawn.
      </Typography>

      <ChoiceFormRow
        label={"Face size"}
        options={LEVEL_CUBE_SIZES.map((side: number) => ({ value: String(side), label: formatCount(side) }))}
        value={String(size)}
        onChange={onSize}
      />
      <Typography className={"text-text-secondary"} variant={"caption"}>
        {`${formatCount(size)} pixels a face · saved as a ${formatCount(crossWidth)} × ${formatCount(crossHeight)} cross`}
      </Typography>

      <ChoiceFormRow
        label={"Supersampling"}
        description={"Each face drawn times over and reduced, which keeps wires and foliage from breaking up."}
        options={LEVEL_CUBE_SUPERSAMPLINGS.map((times: number) => ({
          value: String(times),
          label: `${times}×`,
          isDisabled: !isLevelCubeSupersamplingDrawn(size, times),
        }))}
        value={String(supersampling)}
        onChange={onSupersampling}
      />
      <ChoiceFormRow
        label={"Settle frames"}
        description={"Frames each face is drawn before the one kept, for the temporal passes to converge."}
        options={LEVEL_CAPTURE_SETTLE_FRAMES.map((frames: number) => ({
          value: String(frames),
          label: String(frames),
        }))}
        value={String(settleFrames)}
        onChange={onSettleFrames}
      />
    </div>
  );
}
