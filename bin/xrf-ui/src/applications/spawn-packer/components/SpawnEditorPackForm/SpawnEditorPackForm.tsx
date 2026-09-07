import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect } from "react";

import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { EApplicationId } from "@/core/routing/application";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { SpawnConversionOutcome } from "@/core/spawn/components/SpawnConversionOutcome";
import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";

/**
 * Build a packed spawn file from chunks on disk.
 */
export function SpawnEditorPackForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const conversionService: SpawnConversionService = useInjection(SpawnConversionService);
  const isLoading: boolean = conversionService.operation.isRunning;

  const source: IPathField = usePathField({
    application: EApplicationId.SPAWN_PACKER,
    id: "source",
    title: "Select unpacked spawn directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed: () => resolveOutputPath(EApplicationId.SPAWN_UNPACKER),
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.SPAWN_PACKER,
    id: "destination",
    title: "Select spawn file output",
    filters: [{ name: "spawn", extensions: ["spawn"] }],
    isSave: true,
    isDisabled: isLoading,
    seed: () => resolveOutputPath(EApplicationId.SPAWN_PACKER, "all.spawn"),
  });

  const onPack = useCallback(async () => {
    if (!source.value || !destination.value) {
      return log.error("Cannot pack spawn file, expected correct paths");
    }

    await conversionService.pack(source.value, destination.value);
  }, [conversionService, destination.value, log, source.value]);

  useEffect(() => {
    conversionService.operation.reset();
  }, [conversionService, source.value, destination.value]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!source.isValid || !destination.isValid}
      title={"Pack spawn file"}
      description={
        "Builds one spawn file from the unpacked chunks. The output file is overwritten. " +
        "Cancellation stops before writing; a write already started finishes."
      }
      error={conversionService.operation.error ?? undefined}
      submitLabel={"Pack"}
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
      onSubmit={onPack}
    >
      <PathFormRow
        isDisabled={isLoading}
        label={"Source"}
        description={"Directory holding the unpacked spawn chunks"}
        field={source}
      />

      <PathFormRow
        isDisabled={isLoading}
        label={"Output spawn"}
        description={"Where the packed *.spawn file is written"}
        field={destination}
      />
    </PickerForm>
  );
}
