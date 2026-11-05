import { useInjection } from "@wirestate/react";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { EditorPopoverContent } from "@xrf/ui/editor/popover-content";
import {
  EditorPopoverNote,
  EditorPopoverRow,
  EditorPopoverSwitchRow,
  IEditorPopoverRowControlProps,
} from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { AddCircleIcon as AddIcon } from "@xrf/ui/icons/add-circle";
import { GridOnIcon as GridIcon } from "@xrf/ui/icons/grid-on";
import { RemoveCircleIcon as RemoveIcon } from "@xrf/ui/icons/remove-circle";
import { ReactElement, useCallback } from "react";

import { ENGINE_GRID_SQUARE } from "@/core/sprite-equipment/lib";
import { SpriteEquipmentEditorService } from "@/tools/sprite-equipment-editor/services/sprite-equipment-editor";

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

  const onReset = useCallback(() => spriteEquipmentService.setGridSize(ENGINE_GRID_SQUARE), [spriteEquipmentService]);

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
      <EditorPopoverContent
        label={"Grid"}
        resetLabel={`Reset to the engine's ${ENGINE_GRID_SQUARE}px`}
        onReset={isEngineSize ? undefined : onReset}
      >
        <EditorPopoverSwitchRow label={"Show cells"} isChecked={isGridVisible} onChange={onToggle} />

        <EditorPopoverRow label={"Cell size"}>
          {(control: IEditorPopoverRowControlProps) => (
            <div {...control} role={"group"} className={"flex items-center gap-1"}>
              <EditorIconAction
                label={"Decrease grid size"}
                description={`Decrease the grid cell size by ${GRID_SIZE_STEP} pixels`}
                icon={<RemoveIcon />}
                onClick={onDecrease}
              />

              <span className={"min-w-10 text-center text-xs"}>{`${gridSize}px`}</span>

              <EditorIconAction
                label={"Increase grid size"}
                description={`Increase the grid cell size by ${GRID_SIZE_STEP} pixels`}
                icon={<AddIcon />}
                onClick={onIncrease}
              />
            </div>
          )}
        </EditorPopoverRow>

        <EditorPopoverNote>
          {isEngineSize
            ? `${ENGINE_GRID_SQUARE}px is what the engine reads, so a cell here is one inv_grid unit.`
            : `The engine reads ${ENGINE_GRID_SQUARE}px cells. At this size the lattice is a guide, and the readout no longer matches inv_grid_x.`}
        </EditorPopoverNote>
      </EditorPopoverContent>
    </EditorPopoverAction>
  );
}
