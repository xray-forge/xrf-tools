import { Box } from "@mui/material";
import { ReactElement } from "react";

import { EquipmentSpriteViewer } from "@/applications/sprite-equipment-editor/components/sprite-view/EquipmentSpriteViewer";
import { BaseComponentProps } from "@/lib/dom/element-types";

export function EquipmentSpriteEditorWorkspace({
  "data-testid": dataTestId = "equipment-sprite-editor-workspace",
  id,
  className,
}: BaseComponentProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className ? `workspace ${className}` : "workspace"}
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
