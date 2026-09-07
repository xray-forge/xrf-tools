import { default as AddIcon } from "@mui/icons-material/AddCircle";
import { default as RemoveIcon } from "@mui/icons-material/RemoveCircle";
import { Box, FormControlLabel, Switch } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";

interface IEquipmentGridControlsProps {
  isGridVisible: boolean;
  gridSize: number;
  onSetGridSize: (size: number) => void;
  onSetGridVisibility: (isVisible: boolean) => void;
}

export function EquipmentGridControls({
  isGridVisible,
  gridSize,
  onSetGridSize,
  onSetGridVisibility,
}: IEquipmentGridControlsProps): ReactElement {
  const onGridVisibilityToggled = useCallback(() => {
    onSetGridVisibility(!isGridVisible);
  }, [isGridVisible, onSetGridVisibility]);

  const onDecreaseGridSize = useCallback(() => {
    onSetGridSize(gridSize - 5);
  }, [gridSize, onSetGridSize]);

  const onIncreaseGridSize = useCallback(() => {
    onSetGridSize(gridSize + 5);
  }, [gridSize, onSetGridSize]);

  return (
    <Box sx={{ display: "flex", alignItems: "center", position: "absolute", right: 4, top: 4 }}>
      <FormControlLabel
        label={"Grid"}
        labelPlacement={"start"}
        control={<Switch size={"small"} checked={isGridVisible} onChange={onGridVisibilityToggled} />}
      />

      <Box sx={{ margin: 1 }} />

      <EditorIconAction
        label={"Decrease grid size"}
        description={"Decrease the grid cell size by 5 pixels"}
        icon={<RemoveIcon />}
        onClick={onDecreaseGridSize}
      />

      <Box sx={{ marginLeft: 0.5, marginRight: 0.5 }}>{gridSize}</Box>

      <EditorIconAction
        label={"Increase grid size"}
        description={"Increase the grid cell size by 5 pixels"}
        icon={<AddIcon />}
        onClick={onIncreaseGridSize}
      />
    </Box>
  );
}
