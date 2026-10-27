import { FormatListBulletedIcon as ListIcon } from "@xrf/ui/icons/format-list-bulleted";
import { WarningAmberIcon as WarningIcon } from "@xrf/ui/icons/warning-amber";

import { IShellPanel } from "@/core/shell/panels/shell-panel";
import { EquipmentOccupantsPanel } from "@/tools/sprite-equipment-editor/components/panels/EquipmentOccupantsPanel";
import { EquipmentProblemsPanel } from "@/tools/sprite-equipment-editor/components/panels/EquipmentProblemsPanel";

/** The panels the sprite editor offers. */
export const SPRITE_EQUIPMENT_EDITOR_PANELS: Array<IShellPanel> = [
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
