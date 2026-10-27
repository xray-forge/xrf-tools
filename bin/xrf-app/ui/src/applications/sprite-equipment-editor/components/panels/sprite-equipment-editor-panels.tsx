import { FormatListBulletedIcon as ListIcon } from "@xrf/ui/icons/format-list-bulleted";
import { WarningAmberIcon as WarningIcon } from "@xrf/ui/icons/warning-amber";

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
