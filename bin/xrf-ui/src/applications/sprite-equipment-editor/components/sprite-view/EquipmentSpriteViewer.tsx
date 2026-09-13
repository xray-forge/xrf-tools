import { Box, CircularProgress, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import {
  IEquipmentPngDescriptor,
  SpriteEquipmentEditorService,
} from "@/applications/sprite-equipment-editor/services/editor";
import { IEquipmentLayout, toEquipmentLayout } from "@/core/sprite-equipment/lib";
import { IImageViewportView, ImageViewport } from "@/core/ui/media/ImageViewport";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { EquipmentGridCanvas } from "./EquipmentGridCanvas";
import { EquipmentGridControls } from "./EquipmentGridControls";
import { EquipmentGridDetails } from "./EquipmentGridDetails";
import { EquipmentGridMoveOver } from "./EquipmentGridMoveOver";
import { IEquipmentGridSelection, useEquipmentGridSelection } from "./use-equipment-grid-selection";

/**
 * The open sheet, with the lattice drawn over it.
 */
export function EquipmentSpriteViewer({
  "data-testid": dataTestId = "equipment-sprite-viewer",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);

  const sprite: Nullable<IEquipmentPngDescriptor> = spriteEquipmentService.spriteImage.value;
  const isLoading: boolean = spriteEquipmentService.spriteImage.isLoading;
  const isGridVisible: boolean = spriteEquipmentService.isGridVisible;
  const gridSize: number = spriteEquipmentService.gridSize;

  const layout: Nullable<IEquipmentLayout> = useMemo(
    () => (sprite ? toEquipmentLayout(sprite.image.width, sprite.image.height, gridSize, sprite.descriptors) : null),
    [gridSize, sprite]
  );

  const selection: IEquipmentGridSelection = useEquipmentGridSelection(layout);

  const renderOverlay = useCallback(
    (view: IImageViewportView): Nullable<ReactElement> =>
      layout ? (
        <EquipmentGridCanvas
          view={view}
          layout={layout}
          isGridVisible={isGridVisible}
          hoveredCell={selection.hoveredCell}
          selectedCell={selection.selectedCell}
        />
      ) : null,
    [isGridVisible, layout, selection.hoveredCell, selection.selectedCell]
  );

  if (!sprite || !layout) {
    return (
      <Box
        data-testid={dataTestId}
        id={id}
        className={className}
        sx={{ display: "flex", alignItems: "center", justifyContent: "center", width: "100%", height: "100%" }}
      >
        {isLoading ? (
          <CircularProgress size={28} />
        ) : (
          <Typography variant={"body2"} color={"text.secondary"}>
            No sprite open
          </Typography>
        )}
      </Box>
    );
  }

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ position: "relative", display: "flex", width: "100%", height: "100%", minWidth: 0, minHeight: 0 }}
    >
      <ImageViewport
        src={sprite.image.src}
        alt={sprite.name}
        width={sprite.image.width}
        height={sprite.image.height}
        renderOverlay={renderOverlay}
        onContentPointerMove={selection.onPointerMove}
        onContentClick={selection.onClick}
      />

      {selection.selectedCell ? (
        <EquipmentGridDetails cell={selection.selectedCell} layout={layout} onClose={selection.onClearSelection} />
      ) : null}

      {selection.hoveredCell ? <EquipmentGridMoveOver cell={selection.hoveredCell} /> : null}

      <EquipmentGridControls
        gridSize={gridSize}
        isGridVisible={isGridVisible}
        onSetGridSize={spriteEquipmentService.setGridSize}
        onSetGridVisibility={spriteEquipmentService.setGridVisibility}
      />
    </Box>
  );
}
