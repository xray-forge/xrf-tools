import { default as AddIcon } from "@mui/icons-material/AddCircle";
import { default as RemoveIcon } from "@mui/icons-material/RemoveCircle";
import { Box } from "@mui/material";
import { ReactElement } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";

interface IEquipmentGridZoomProps {
  zoom: number;
  onZoomUp: () => void;
  onZoomDown: () => void;
}

export function EquipmentGridZoom({ zoom, onZoomUp, onZoomDown }: IEquipmentGridZoomProps): ReactElement {
  return (
    <Box sx={{ display: "flex", alignItems: "center", position: "absolute", right: 4, bottom: 4 }}>
      <EditorIconAction
        label={"Zoom out"}
        description={"Decrease sprite magnification"}
        icon={<RemoveIcon />}
        onClick={onZoomDown}
      />

      <Box sx={{ marginLeft: 0.5, marginRight: 0.5 }}>{zoom.toFixed(2)}</Box>

      <EditorIconAction
        label={"Zoom in"}
        description={"Increase sprite magnification"}
        icon={<AddIcon />}
        onClick={onZoomUp}
      />
    </Box>
  );
}
