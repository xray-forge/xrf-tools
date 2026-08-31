import { Box } from "@mui/material";
import { SxProps } from "@mui/system";
import { memo, ReactElement, useMemo } from "react";

import { GridMapper, TEquipmentCell } from "@/core/sprite-equipment";
import { Nullable } from "@/lib/types/general";

interface IEquipmentSpriteGridProps {
  isGridVisible: boolean;
  selectedCell: Nullable<TEquipmentCell>;
  gridMapper: GridMapper;
  onCellSelected: (row: number, column: number) => void;
  onCellMovedOver: (row: number, column: number) => void;
}

export const EquipmentSpriteGrid = memo(
  ({
    selectedCell,
    isGridVisible,
    gridMapper,
    onCellSelected,
    onCellMovedOver,
  }: IEquipmentSpriteGridProps): ReactElement => {
    const sx: SxProps = useMemo(
      () => ({
        userSelect: "none",
        border: isGridVisible ? "1px solid" : "none",
        borderColor: "rgba(2,2,2,0.6)",
        "&.selected": {
          background: "rgba(39,48,117,0.49)",
        },
        "&.selected:hover": {
          background: "rgba(33,77,172,0.49)",
        },
        "&:hover": {
          background: "rgba(0,0,0,0.49)",
        },
        "&:hover .coordinates": {
          display: "flex",
          userSelect: "none",
        },
      }),
      [isGridVisible]
    );

    return (
      <Box sx={{ position: "absolute", left: 0, top: 0, right: 0, bottom: 0, bgcolor: "#66666608" }}>
        {gridMapper.grid.map((row, rowIndex) => (
          <Box key={rowIndex} sx={{ display: "flex" }}>
            {row.map((column, columnIndex) => (
              <Box
                key={columnIndex}
                className={
                  selectedCell && selectedCell[0] === rowIndex && selectedCell[1] === columnIndex ? "selected" : ""
                }
                onClick={() => onCellSelected(rowIndex, columnIndex)}
                onMouseMove={() => onCellMovedOver(rowIndex, columnIndex)}
                sx={[
                  {
                    display: "flex",
                    flexWrap: "nowrap",
                    justifyContent: "center",
                    alignItems: "center",
                    minHeight: gridMapper.gridSize,
                    maxHeight: gridMapper.gridSize,
                    minWidth: gridMapper.gridSize,
                    maxWidth: gridMapper.gridSize,
                  },
                  sx,
                ]}
              >
                <Box className={"coordinates"} sx={{ display: "none" }}>
                  {columnIndex}:{rowIndex}: ({column?.length ?? 0})
                </Box>
              </Box>
            ))}
          </Box>
        ))}
      </Box>
    );
  }
);

EquipmentSpriteGrid.displayName = "EquipmentSpriteGrid";
