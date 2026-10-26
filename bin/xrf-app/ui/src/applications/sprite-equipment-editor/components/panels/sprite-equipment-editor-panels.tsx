import { default as ListIcon } from "@mui/icons-material/FormatListBulleted";
import { default as WarningIcon } from "@mui/icons-material/WarningAmber";

import { EquipmentOccupantsPanel } from "@/applications/sprite-equipment-editor/components/panels/EquipmentOccupantsPanel";
import { EquipmentProblemsPanel } from "@/applications/sprite-equipment-editor/components/panels/EquipmentProblemsPanel";
import { IEditorPanel } from "@/core/shell/editor-shell";

/** The panels the sprite editor offers. */
export const SPRITE_EQUIPMENT_EDITOR_PANELS: Array<IEditorPanel> = [
  {
    icon: <ListIcon />,
    id: "occupants",
    isOpenByDefault: true,
    label: "Occupants",
    render: () => <EquipmentOccupantsPanel />,
  },
  {
    icon: <WarningIcon />,
    id: "problems",
    label: "Problems",
    render: () => <EquipmentProblemsPanel />,
  },
];
