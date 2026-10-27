import { useInjection } from "@wirestate/react";
import { IPanZoomPoint } from "@xrf/math";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CircularProgress, Typography } from "@xrf/ui/material";
import { IImageViewportView, ImageViewport } from "@xrf/ui/media/ImageViewport";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement, useCallback } from "react";

import { EquipmentGridService } from "@/applications/sprite-equipment-editor/services/equipment-grid";
import {
  IOpenEquipmentSprite,
  SpriteEquipmentEditorService,
} from "@/applications/sprite-equipment-editor/services/sprite-equipment-editor";
import { describeEquipmentSheet, IEquipmentLayout, toCellAt } from "@/core/sprite-equipment/lib";

import { EquipmentGridCanvas } from "./EquipmentGridCanvas";
import { EquipmentGridMoveOver } from "./EquipmentGridMoveOver";
import { IEquipmentGridHover, useEquipmentGridHover } from "./use-equipment-grid-hover";

/**
 * The open sheet, with the lattice drawn over it.
 */
export function EquipmentSpriteViewer({
  "data-testid": dataTestId = "equipment-sprite-viewer",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);
  const gridService: EquipmentGridService = useInjection(EquipmentGridService);

  const sprite: Nullable<IOpenEquipmentSprite> = spriteEquipmentService.spriteImage.value;
  const isLoading: boolean = spriteEquipmentService.spriteImage.isLoading;
  const isGridVisible: boolean = spriteEquipmentService.isGridVisible;
  const isOccupancyVisible: boolean = spriteEquipmentService.isOccupancyVisible;

  const layout: Nullable<IEquipmentLayout> = gridService.layout;
  const hover: IEquipmentGridHover = useEquipmentGridHover(layout);

  const onClick = useCallback(
    (point: IPanZoomPoint): void => gridService.selectCell(layout ? toCellAt(layout.grid, point.x, point.y) : null),
    [gridService, layout]
  );

  const renderOverlay = useCallback(
    (view: IImageViewportView): Nullable<ReactElement> =>
      layout ? (
        <EquipmentGridCanvas
          view={view}
          layout={layout}
          isGridVisible={isGridVisible}
          isOccupancyVisible={isOccupancyVisible}
          hoveredCell={hover.hoveredCell}
          selectedCell={gridService.selectedCell}
        />
      ) : null,
    [gridService.selectedCell, hover.hoveredCell, isGridVisible, isOccupancyVisible, layout]
  );

  if (!sprite || !layout) {
    return (
      <div data-testid={dataTestId} id={id} className={cn("flex h-full w-full items-center justify-center", className)}>
        {isLoading ? (
          <CircularProgress size={28} />
        ) : (
          <Typography variant={"body2"} color={"text.secondary"}>
            No sprite open
          </Typography>
        )}
      </div>
    );
  }

  return (
    <div data-testid={dataTestId} id={id} className={cn("relative flex h-full min-h-0 w-full min-w-0", className)}>
      <ImageViewport
        src={sprite.image.src}
        alt={describeEquipmentSheet(sprite.metadata.open.sheet)}
        width={sprite.image.width}
        height={sprite.image.height}
        renderOverlay={renderOverlay}
        onContentPointerMove={hover.onPointerMove}
        onContentClick={onClick}
      />

      {hover.hoveredCell ? <EquipmentGridMoveOver cell={hover.hoveredCell} /> : null}
    </div>
  );
}
