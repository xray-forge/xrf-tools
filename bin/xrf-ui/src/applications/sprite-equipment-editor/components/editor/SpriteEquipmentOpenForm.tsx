import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/editor";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { CheckboxFormRow, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";

export function SpriteEquipmentOpenForm({
  "data-testid": dataTestId = "sprite-equipment-open-form",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);

  const isLoading: boolean = spriteEquipmentService.spriteImage.isLoading;

  const sprite: IPathField = usePathField({
    application: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
    id: "sprite",
    title: "Select equipment sprite",
    filters: [{ name: "dds", extensions: ["dds"] }],
    isDisabled: isLoading,
  });

  const systemLtx: IPathField = usePathField({
    application: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
    id: "system-ltx",
    title: "Select system.ltx",
    filters: [{ name: "ltx", extensions: ["ltx"] }],
    isDisabled: isLoading,
  });

  // Opt-in rather than detected: a patched Anomaly tree and a vanilla one look alike, and resolving one under the
  // other's rules answers wrong icon descriptors rather than failing. Remembered for the session, because reopening
  // takes no arguments and has to answer the same values.
  const [isDltx, setDltx] = useState<boolean>(false);

  const onOpenEquipmentClicked = useCallback(() => {
    if (sprite.value && systemLtx.value) {
      spriteEquipmentService.openEquipmentProject(sprite.value, systemLtx.value, isDltx);
    } else {
      log.info("Cannot open equipment editor without every path");
    }
  }, [spriteEquipmentService, log, sprite.value, systemLtx.value, isDltx]);

  return (
    <PickerForm
      data-testid={dataTestId}
      id={id}
      className={className}
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

      <CheckboxFormRow
        label={"DLTX"}
        description={"Use DLTX patch rules when reading the system configuration"}
        isChecked={isDltx}
        isDisabled={isLoading}
        onChange={setDltx}
      />
    </PickerForm>
  );
}
