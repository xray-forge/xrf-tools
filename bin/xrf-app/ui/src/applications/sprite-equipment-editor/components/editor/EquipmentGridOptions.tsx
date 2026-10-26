import { default as AddIcon } from "@mui/icons-material/AddCircle";
import { default as GridIcon } from "@mui/icons-material/GridOn";
import { default as RemoveIcon } from "@mui/icons-material/RemoveCircle";
import { FormControlLabel, Switch, Typography } from "@mui/material";
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
      <div className={"flex flex-col gap-2 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Grid
        </Typography>

        <FormControlLabel
          label={"Show cells"}
          control={<Switch size={"small"} checked={isGridVisible} onChange={onToggle} />}
        />

        <Typography className={"text-text-secondary"} variant={"overline"}>
          Cell size
        </Typography>

        <div className={"flex items-center gap-1"}>
          <EditorIconAction
            label={"Decrease grid size"}
            description={`Decrease the grid cell size by ${GRID_SIZE_STEP} pixels`}
            icon={<RemoveIcon />}
            onClick={onDecrease}
          />

          <Typography className={"min-w-10 text-center"} variant={"body2"}>
            {`${gridSize}px`}
          </Typography>

          <EditorIconAction
            label={"Increase grid size"}
            description={`Increase the grid cell size by ${GRID_SIZE_STEP} pixels`}
            icon={<AddIcon />}
            onClick={onIncrease}
          />
        </div>

        <Typography className={"max-w-60 text-text-secondary"} variant={"caption"}>
          {isEngineSize
            ? `${ENGINE_GRID_SQUARE}px is what the engine reads, so a cell here is one inv_grid unit.`
            : `The engine reads ${ENGINE_GRID_SQUARE}px cells. At this size the lattice is a guide, and the readout no longer matches inv_grid_x.`}
        </Typography>
      </div>
    </EditorPopoverAction>
  );
}
