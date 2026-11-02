import { useInjection } from "@wirestate/react";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback } from "react";

import { SPAWN_FILE_FILTERS } from "@/core/path/file-filters";
import { EToolId } from "@/core/routing/tool";
import { PickerForm } from "@/core/shell/chrome/PickerForm";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { SpawnFileService } from "@/tools/spawn-editor/services/spawn-file.service";

export function SpawnEditorOpenForm({
  "data-testid": dataTestId = "spawn-editor-open-form",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  const isLoading: boolean = spawnFileService.isOpening;

  const spawn: IPathField = usePathField({
    tool: EToolId.SPAWN_EDITOR,
    id: "file",
    title: "Select spawn file",
    filters: SPAWN_FILE_FILTERS,
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
      data-testid={dataTestId}
      id={id}
      className={className}
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
