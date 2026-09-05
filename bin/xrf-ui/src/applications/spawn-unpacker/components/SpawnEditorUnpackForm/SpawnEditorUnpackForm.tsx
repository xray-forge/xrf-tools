import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect } from "react";

import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole, resolveOutputPath } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { SpawnConversionOutcome } from "@/core/spawn/components/SpawnConversionOutcome";
import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";

/**
 * Expand a packed spawn file into chunks on disk.
 */
export function SpawnEditorUnpackForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const pathsService: PathsService = useInjection(PathsService);
  const conversionService: SpawnConversionService = useInjection(SpawnConversionService);
  const isLoading: boolean = conversionService.operation.isRunning;

  const source: IPathField = usePathField({
    application: EApplicationId.SPAWN_UNPACKER,
    id: "source",
    title: "Select spawn file",
    filters: [{ name: "spawn", extensions: ["spawn"] }],
    isDisabled: isLoading,
    seed: () => resolveExistingPathRole(EPathRole.ALL_SPAWN, pathsService.paths),
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.SPAWN_UNPACKER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed: () => resolveOutputPath(EApplicationId.SPAWN_UNPACKER, pathsService.paths),
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
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!source.isValid || !destination.isValid}
      title={"Unpack spawn file"}
      description={
        "Writes the file's chunks into the destination directory, replacing files of the same name. " +
        "Cancellation stops before writing; a write already started finishes."
      }
      error={conversionService.operation.error ?? undefined}
      submitLabel={"Unpack"}
      status={
        conversionService.operation.job ? (
          <JobProgressView job={conversionService.operation.job} onCancel={conversionService.operation.cancel} />
        ) : null
      }
      result={
        conversionService.operation.result ? (
          <SpawnConversionOutcome result={conversionService.operation.result} />
        ) : null
      }
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
    </PickerForm>
  );
}
