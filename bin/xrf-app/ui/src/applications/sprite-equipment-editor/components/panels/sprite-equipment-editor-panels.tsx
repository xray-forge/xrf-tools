import { FormatListBulletedIcon as ListIcon, WarningAmberIcon as WarningIcon } from "@xrf/ui/icons";

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
