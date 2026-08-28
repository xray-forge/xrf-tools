import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { SpawnFileService } from "@/core/spawn/services";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";

export function SpawnEditorOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);
  const pathsService: PathsService = useInjection(PathsService);

  const isLoading: boolean = spawnFileService.header.isLoading;

  const spawn: IPathField = usePathField({
    application: EApplicationId.SPAWN_EDITOR,
    id: "file",
    title: "Select spawn file",
    filters: [{ name: "spawn", extensions: ["spawn"] }],
    isDisabled: isLoading,
    seed: () => resolveExistingPathRole(EPathRole.ALL_SPAWN, pathsService.paths),
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
      error={spawnFileService.header.error ? String(spawnFileService.header.error) : undefined}
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
