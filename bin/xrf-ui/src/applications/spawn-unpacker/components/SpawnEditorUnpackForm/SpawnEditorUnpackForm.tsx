import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect } from "react";

import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { SPAWN_FILE_FILTERS } from "@/core/path/file-filters";
import { EApplicationId } from "@/core/routing/application";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { SpawnConversionOutcome } from "@/core/spawn/components/SpawnConversionOutcome";
import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";

/**
 * Expand a packed spawn file into chunks on disk.
 */
export function SpawnEditorUnpackForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const conversionService: SpawnConversionService = useInjection(SpawnConversionService);
  const isLoading: boolean = conversionService.operation.isRunning;

  const source: IPathField = usePathField({
    application: EApplicationId.SPAWN_UNPACKER,
    id: "source",
    title: "Select spawn file",
    filters: SPAWN_FILE_FILTERS,
    isDisabled: isLoading,
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.SPAWN_UNPACKER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed: () => resolveOutputPath(EApplicationId.SPAWN_UNPACKER),
  });

  const onUnpack = useCallback(async () => {
    if (!source.value || !destination.value) {
      return log.error("Cannot unpack spawn file, expected correct paths");
    }

    await conversionService.unpack(source.value, destination.value);
  }, [conversionService, destination.value, log, source.value]);

  useEffect(() => {
    conversionService.operation.reset();
  }, [conversionService, source.value, destination.value]);

  return (
    <JobPickerForm
      operation={conversionService.operation}
      isSubmitDisabled={!source.isValid || !destination.isValid}
      title={"Unpack spawn file"}
      description={
        "Writes the file's chunks into the destination directory, replacing files of the same name. " +
        "Cancellation stops before writing; a write already started finishes."
      }
      submitLabel={"Unpack"}
      renderResult={(result) => <SpawnConversionOutcome result={result} />}
      onSubmit={onUnpack}
    >
      <PathFormRow
        isDisabled={isLoading}
        label={"Source"}
        description={"The packed *.spawn file to read"}
        field={source}
      />

      <PathFormRow
        isDisabled={isLoading}
        label={"Destination"}
        description={"Directory the unpacked chunks are written to"}
        field={destination}
      />
    </JobPickerForm>
  );
}
