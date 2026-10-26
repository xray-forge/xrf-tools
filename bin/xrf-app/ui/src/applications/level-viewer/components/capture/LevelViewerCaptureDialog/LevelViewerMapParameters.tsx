import { Button, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatCount, formatNumber } from "@xrf/format";
import { ReactElement, useCallback } from "react";

import { LEVEL_CAPTURE_SETTLE_FRAMES } from "@/applications/level-viewer/lib/capture/level-capture";
import {
  ELevelMapArea,
  ELevelMapLook,
  ILevelMapParameters,
  ILevelMapRect,
  isLevelMapSupersamplingDrawn,
  LEVEL_MAP_LONG_SIDES,
  LEVEL_MAP_SUPERSAMPLINGS,
  toLevelMapMetresPerPixel,
  toLevelMapTileCount,
} from "@/applications/level-viewer/lib/map/level-map-capture";
import { LevelViewerCaptureService } from "@/applications/level-viewer/services/capture";
import { cn } from "@/core/theme/class-name";
import { CheckboxFormRow, ChoiceFormRow, FormRow, IChoiceFormRowOption } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { LevelViewerMapNumberField } from "./LevelViewerMapNumberField";

interface ILevelViewerMapParametersProps extends BaseComponentProps {
  parameters: ILevelMapParameters;
}

/**
 * What a level map is captured with: its area and where it comes from, the heights it is cut between, its size, its
 * look and what is drawn, and how hard each tile is drawn.
 */
export function LevelViewerMapParameters({
  "data-testid": dataTestId = "level-viewer-map-parameters",
  id,
  className,
  parameters,
}: ILevelViewerMapParametersProps): ReactElement {
  const captureService: LevelViewerCaptureService = useInjection(LevelViewerCaptureService);
  const {
    area,
    rect,
    bottom,
    top,
    longSide,
    look,
    isSpawned,
    isWatered,
    isShadowed,
    isGrassed,
    supersampling,
    settleFrames,
  } = parameters;
  const [width, height]: [number, number] = captureService.mapSize ?? [longSide, longSide];
  const tiles: number = toLevelMapTileCount(width, height);
  const isCustom: boolean = area === ELevelMapArea.CUSTOM;
  const areas: ReadonlyArray<IChoiceFormRowOption<ELevelMapArea>> = [
    { value: ELevelMapArea.LEVEL_MAP, label: "Level map", isDisabled: captureService.levelMapRect === null },
    { value: ELevelMapArea.LEVEL_GEOMETRY, label: "Level geometry" },
    { value: ELevelMapArea.CUSTOM, label: "Custom" },
  ];

  const onArea = useCallback((value: ELevelMapArea) => captureService.setArea(value), [captureService]);

  const onRect = useCallback(
    (key: keyof ILevelMapRect, value: number) =>
      captureService.setMapParameters({ area: ELevelMapArea.CUSTOM, rect: { ...rect, [key]: value } }),
    [captureService, rect]
  );

  const onMinX = useCallback((value: number) => onRect("minX", value), [onRect]);

  const onMinZ = useCallback((value: number) => onRect("minZ", value), [onRect]);

  const onMaxX = useCallback((value: number) => onRect("maxX", value), [onRect]);

  const onMaxZ = useCallback((value: number) => onRect("maxZ", value), [onRect]);

  const onBottom = useCallback((value: number) => captureService.setMapParameters({ bottom: value }), [captureService]);

  const onTop = useCallback((value: number) => captureService.setMapParameters({ top: value }), [captureService]);

  const onLongSide = useCallback(
    (value: string) => captureService.setMapParameters({ longSide: Number(value) }),
    [captureService]
  );

  const onLook = useCallback(
    (value: ELevelMapLook) =>
      // The engine's map draws the level alone; the view's own look draws what the view does.
      captureService.setMapParameters({ look: value, isSpawned: value === ELevelMapLook.CURRENT_VIEW }),
    [captureService]
  );

  const onSpawned = useCallback(
    (value: boolean) => captureService.setMapParameters({ isSpawned: value }),
    [captureService]
  );

  const onWatered = useCallback(
    (value: boolean) => captureService.setMapParameters({ isWatered: value }),
    [captureService]
  );

  const onShadowed = useCallback(
    (value: boolean) => captureService.setMapParameters({ isShadowed: value }),
    [captureService]
  );

  const onGrassed = useCallback(
    (value: boolean) => captureService.setMapParameters({ isGrassed: value }),
    [captureService]
  );

  const onSupersampling = useCallback(
    (value: string) => captureService.setMapParameters({ supersampling: Number(value) }),
    [captureService]
  );

  const onSettleFrames = useCallback(
    (value: string) => captureService.setMapParameters({ settleFrames: Number(value) }),
    [captureService]
  );

  const onFromView = useCallback(() => captureService.takeAreaFromView(), [captureService]);

  const onFromCamera = useCallback(() => captureService.takeTopFromCamera(), [captureService]);

  const onFullHeight = useCallback(() => captureService.resetHeights(), [captureService]);

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-col gap-4", className)}>
      <ChoiceFormRow
        label={"Area"}
        description={
          "Level map is where the PDA lays the level's map; Level geometry the box every sector covers, as the engine " +
          "falls back to; Custom four numbers of your own."
        }
        options={areas}
        value={area}
        onChange={onArea}
      />

      <FormRow label={"Area, metres"} isGroup={true}>
        <div className={"grid grid-cols-4 gap-2"}>
          <LevelViewerMapNumberField label={"Least x"} value={rect.minX} isDisabled={!isCustom} onCommit={onMinX} />
          <LevelViewerMapNumberField label={"Least z"} value={rect.minZ} isDisabled={!isCustom} onCommit={onMinZ} />
          <LevelViewerMapNumberField label={"Most x"} value={rect.maxX} isDisabled={!isCustom} onCommit={onMaxX} />
          <LevelViewerMapNumberField label={"Most z"} value={rect.maxZ} isDisabled={!isCustom} onCommit={onMaxZ} />
        </div>
        <Button className={"self-start"} size={"small"} onClick={onFromView}>
          From current view
        </Button>
      </FormRow>

      <FormRow
        label={"Heights, metres"}
        description={"What stands below the bottom or above the top is cut away, as a floor of a building is drawn."}
        isGroup={true}
      >
        <div className={"grid grid-cols-4 gap-2"}>
          <LevelViewerMapNumberField label={"Bottom"} value={bottom} onCommit={onBottom} />
          <LevelViewerMapNumberField label={"Top"} value={top} onCommit={onTop} />
          <Button size={"small"} onClick={onFromCamera}>
            Top at camera
          </Button>
          <Button size={"small"} onClick={onFullHeight}>
            Full height
          </Button>
        </div>
      </FormRow>

      <ChoiceFormRow
        label={"Size"}
        options={LEVEL_MAP_LONG_SIDES.map((side: number) => ({ value: String(side), label: formatCount(side) }))}
        value={String(longSide)}
        onChange={onLongSide}
      />
      <Typography className={"text-text-secondary"} variant={"caption"}>
        {`${formatCount(width)} × ${formatCount(height)} pixels · ` +
          `${formatNumber(toLevelMapMetresPerPixel(rect, longSide), 2)} m a pixel · ` +
          `${formatCount(tiles)} ${tiles === 1 ? "tile" : "tiles"}`}
      </Typography>

      <ChoiceFormRow
        label={"Look"}
        description={
          "Map draws the level as the engine's level map does: its map weather and static geometry. Current view " +
          "keeps the view's weather, time and lighting. Both leave out what the engine's map does: the sky, fog, " +
          "particles and, unless asked for below, the grass."
        }
        options={[
          { value: ELevelMapLook.MAP, label: "Map" },
          { value: ELevelMapLook.CURRENT_VIEW, label: "Current view" },
        ]}
        value={look}
        onChange={onLook}
      />
      <CheckboxFormRow label={"Spawned objects"} isChecked={isSpawned} onChange={onSpawned} />
      <CheckboxFormRow label={"Water"} isChecked={isWatered} onChange={onWatered} />
      <CheckboxFormRow label={"Shadows"} isChecked={isShadowed} onChange={onShadowed} />
      <CheckboxFormRow
        label={"Grass (experiment)"}
        description={
          "Planted over every tile alike, as far as its corners; the engine's map draws none, and the widest tiles " +
          "plant it thinner."
        }
        isChecked={isGrassed}
        onChange={onGrassed}
      />

      <ChoiceFormRow
        label={"Supersampling"}
        description={"Each tile drawn times over and reduced, which keeps wires and foliage from breaking up."}
        options={LEVEL_MAP_SUPERSAMPLINGS.map((times: number) => ({
          value: String(times),
          label: `${times}×`,
          isDisabled: !isLevelMapSupersamplingDrawn(longSide, times),
        }))}
        value={String(supersampling)}
        onChange={onSupersampling}
      />
      <ChoiceFormRow
        label={"Settle frames"}
        description={"Frames each tile is drawn before the one kept, for the temporal passes to converge."}
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
