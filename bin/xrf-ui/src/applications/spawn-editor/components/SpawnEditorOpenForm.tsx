import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { SpawnFileService } from "@/core/spawn/services";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";

export function SpawnEditorOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  const isLoading: boolean = spawnFileService.isOpening;

  const spawn: IPathField = usePathField({
    application: EApplicationId.SPAWN_EDITOR,
    id: "file",
    title: "Select spawn file",
    filters: [{ name: "spawn", extensions: ["spawn"] }],
    isDisabled: isLoading,
  });

  const onOpen = useCallback(() => {
    if (spawn.value) {
      spawnFileService.openFile(spawn.value);
    } else {
      log.info("Cannot parse spawn file without path");
    }
  }, [log, spawnFileService, spawn.value]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!spawn.isValid}
      title={"Open spawn file"}
      description={"Reads the file into the editor. Nothing is written until you save."}
      error={spawnFileService.chunks.header.error ? String(spawnFileService.chunks.header.error) : undefined}
      submitLabel={"Open"}
      onSubmit={onOpen}
    >
      <PathFormRow
        isDisabled={isLoading}
        label={"Spawn file"}
        description={"The *.spawn file to read into the editor"}
        field={spawn}
      />
    </PickerForm>
  );
}
