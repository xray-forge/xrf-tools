import { default as AddIcon } from "@mui/icons-material/AddCircle";
import { default as GridIcon } from "@mui/icons-material/GridOn";
import { default as RemoveIcon } from "@mui/icons-material/RemoveCircle";
import { Box, FormControlLabel, Switch, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/editor";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { ENGINE_GRID_SQUARE } from "@/core/sprite-equipment/lib";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** How much one step moves the cell size, in sheet pixels. */
const GRID_SIZE_STEP: number = 5;

/**
 * The lattice drawn over the sheet: whether it is shown, and how big its cells are.
 */
export function EquipmentGridOptions({
  "data-testid": dataTestId = "equipment-grid-options",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);

  const isGridVisible: boolean = spriteEquipmentService.isGridVisible;
  const gridSize: number = spriteEquipmentService.gridSize;
  const isEngineSize: boolean = gridSize === ENGINE_GRID_SQUARE;

  const onToggle = useCallback(
    () => spriteEquipmentService.setGridVisibility(!isGridVisible),
    [isGridVisible, spriteEquipmentService]
  );

  const onDecrease = useCallback(
    () => spriteEquipmentService.setGridSize(gridSize - GRID_SIZE_STEP),
    [gridSize, spriteEquipmentService]
  );

  const onIncrease = useCallback(
    () => spriteEquipmentService.setGridSize(gridSize + GRID_SIZE_STEP),
    [gridSize, spriteEquipmentService]
  );

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Grid"}
      description={isGridVisible ? `Grid: ${gridSize}px cells` : "Grid: hidden"}
      icon={<GridIcon />}
      // A size the engine never reads is worth noticing from the toolbar, without opening anything.
      isActive={!isEngineSize}
    >
      <Box sx={{ display: "flex", flexDirection: "column", gap: 1, paddingX: 2, paddingY: 1 }}>
        <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
          Grid
        </Typography>

        <FormControlLabel
          label={"Show cells"}
          control={<Switch size={"small"} checked={isGridVisible} onChange={onToggle} />}
        />

        <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
          Cell size
        </Typography>

        <Box sx={{ display: "flex", alignItems: "center", gap: 0.5 }}>
          <EditorIconAction
            label={"Decrease grid size"}
            description={`Decrease the grid cell size by ${GRID_SIZE_STEP} pixels`}
            icon={<RemoveIcon />}
            onClick={onDecrease}
          />

          <Typography variant={"body2"} sx={{ minWidth: 40, textAlign: "center" }}>
            {`${gridSize}px`}
          </Typography>

          <EditorIconAction
            label={"Increase grid size"}
            description={`Increase the grid cell size by ${GRID_SIZE_STEP} pixels`}
            icon={<AddIcon />}
            onClick={onIncrease}
          />
        </Box>

        <Typography variant={"caption"} sx={{ color: "text.secondary", maxWidth: 240 }}>
          {isEngineSize
            ? `${ENGINE_GRID_SQUARE}px is what the engine reads, so a cell here is one inv_grid unit.`
            : `The engine reads ${ENGINE_GRID_SQUARE}px cells. At this size the lattice is a guide, and the readout no longer matches inv_grid_x.`}
        </Typography>
      </Box>
    </EditorPopoverAction>
  );
}
