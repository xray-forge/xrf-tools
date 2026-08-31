import { Box } from "@mui/material";
import { ReactElement } from "react";

import { EquipmentSpriteViewer } from "@/applications/sprite-equipment-editor/components/sprite-view/EquipmentSpriteViewer";

export function EquipmentSpriteEditorWorkspace(): ReactElement {
  return (
    <Box
      className={"workspace"}
      sx={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        maxWidth: "100%",
        maxHeight: "100%",
        flexGrow: 1,
      }}
    >
      <EquipmentSpriteViewer />
    </Box>
  );
}
