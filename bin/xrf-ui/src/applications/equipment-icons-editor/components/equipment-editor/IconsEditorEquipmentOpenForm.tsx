import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { EquipmentService } from "@/core/equipment-icons";
import { EApplicationId } from "@/core/routing/application";
import { getPathIfExists, getProjectEquipmentDDSPath, getProjectSystemLtxPath } from "@/core/settings/lib/path";
import { ProjectService } from "@/core/settings/services/project";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";

export function IconsEditorEquipmentOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const projectService: ProjectService = useInjection(ProjectService);
  const equipmentService: EquipmentService = useInjection(EquipmentService);

  const isLoading: boolean = equipmentService.spriteImage.isLoading;

  const sprite: IPathField = usePathField({
    application: EApplicationId.EQUIPMENT_ICONS_EDITOR,
    id: "sprite",
    title: "Select equipment sprite",
    filters: [{ name: "dds", extensions: ["dds"] }],
    isDisabled: isLoading,
    seed: async () =>
      projectService.xrfProjectPath ? getPathIfExists(getProjectEquipmentDDSPath(projectService.xrfProjectPath)) : null,
  });

  const systemLtx: IPathField = usePathField({
    application: EApplicationId.EQUIPMENT_ICONS_EDITOR,
    id: "system-ltx",
    title: "Select system.ltx",
    filters: [{ name: "ltx", extensions: ["ltx"] }],
    isDisabled: isLoading,
    seed: async () =>
      projectService.xrfProjectPath ? getPathIfExists(getProjectSystemLtxPath(projectService.xrfProjectPath)) : null,
  });

  const onOpenEquipmentClicked = useCallback(() => {
    if (sprite.value && systemLtx.value) {
      equipmentService.openEquipmentProject(sprite.value, systemLtx.value);
    } else {
      log.info("Cannot open equipment editor without every path");
    }
  }, [equipmentService, log, sprite.value, systemLtx.value]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!sprite.isValid || !systemLtx.isValid}
      title={"Open equipment sprite"}
      description={"Reads the sprite and the configuration that names its icons. Nothing is written."}
      error={equipmentService.spriteImage.error ? String(equipmentService.spriteImage.error) : undefined}
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
