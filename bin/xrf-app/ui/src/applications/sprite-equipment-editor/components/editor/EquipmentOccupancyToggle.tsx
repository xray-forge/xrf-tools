import { useInjection } from "@wirestate/react";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { SelectAllIcon as HighlightIcon } from "@xrf/ui/icons";
import { ReactElement, useCallback } from "react";

import { EquipmentGridService } from "@/applications/sprite-equipment-editor/services/equipment-grid";
import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/sprite-equipment-editor";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";

/**
 * Whether the slots the configuration claims are shaded over the sheet.
 */
export function EquipmentOccupancyToggle({
  "data-testid": dataTestId = "equipment-occupancy-toggle",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);
  const gridService: EquipmentGridService = useInjection(EquipmentGridService);

  const isOn: boolean = spriteEquipmentService.isOccupancyVisible;
  const hasOccupants: boolean = Boolean(gridService.layout?.occupants.length);

  const onToggle = useCallback(
    () => spriteEquipmentService.setOccupancyVisibility(!isOn),
    [isOn, spriteEquipmentService]
  );

  return (
    <EditorViewToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Occupants"}
      description={
        isOn ? "Stop shading the slots the configuration claims" : "Shade the slots the configuration claims"
      }
      icon={<HighlightIcon />}
      isOn={isOn}
      isDisabled={!hasOccupants}
      unavailableTitle={"Nothing is claimed on this sheet"}
      onToggle={onToggle}
    />
  );
}
