import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { SpriteEquipmentService } from "@/core/sprite-equipment";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";

export function SpriteEquipmentOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const pathsService: PathsService = useInjection(PathsService);
  const spriteEquipmentService: SpriteEquipmentService = useInjection(SpriteEquipmentService);

  const isLoading: boolean = spriteEquipmentService.spriteImage.isLoading;

  const sprite: IPathField = usePathField({
    application: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
    id: "sprite",
    title: "Select equipment sprite",
    filters: [{ name: "dds", extensions: ["dds"] }],
    isDisabled: isLoading,
    seed: () => resolveExistingPathRole(EPathRole.EQUIPMENT_SPRITE, pathsService.paths),
  });

  const systemLtx: IPathField = usePathField({
    application: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
    id: "system-ltx",
    title: "Select system.ltx",
    filters: [{ name: "ltx", extensions: ["ltx"] }],
    isDisabled: isLoading,
    seed: () => resolveExistingPathRole(EPathRole.SYSTEM_LTX, pathsService.paths),
  });

  const onOpenEquipmentClicked = useCallback(() => {
    if (sprite.value && systemLtx.value) {
      spriteEquipmentService.openEquipmentProject(sprite.value, systemLtx.value);
    } else {
      log.info("Cannot open equipment editor without every path");
    }
  }, [spriteEquipmentService, log, sprite.value, systemLtx.value]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!sprite.isValid || !systemLtx.isValid}
      title={"Open equipment sprite"}
      description={"Reads the sprite and the configuration that names its icons. Nothing is written."}
      error={spriteEquipmentService.spriteImage.error ? String(spriteEquipmentService.spriteImage.error) : undefined}
      submitLabel={"Open"}
      onSubmit={onOpenEquipmentClicked}
    >
      <PathFormRow
        isDisabled={isLoading}
        label={"Equipment sprite"}
        description={"The packed *.dds holding the inventory icons"}
        field={sprite}
      />

      <PathFormRow
        isDisabled={isLoading}
        label={"System configuration"}
        description={"The system.ltx that names the icons"}
        field={systemLtx}
      />
    </PickerForm>
  );
}
