import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo } from "react";

import { SPAWN_FILE_FILTERS } from "@/core/path/file-filters";
import { EToolId } from "@/core/routing/tool";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { SpawnConversionOutcome } from "@/core/spawn/components/SpawnConversionOutcome";
import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";

interface ISpawnUnpackRequest {
  /** The packed spawn file. */
  source: string;
  /** The directory its chunks are written into. */
  destination: string;
}

/** Extract a packed spawn file into editable chunks. */
export function SpawnUnpackerTool(): ReactElement {
  const conversionService: SpawnConversionService = useInjection(SpawnConversionService);
  const isLoading: boolean = conversionService.operation.isRunning;

  const source: IPathField = usePathField({
    tool: EToolId.SPAWN_UNPACKER,
    id: "source",
    title: "Select spawn file",
    filters: SPAWN_FILE_FILTERS,
    isDisabled: isLoading,
  });

  const destination: IPathField = usePathField({
    tool: EToolId.SPAWN_UNPACKER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed: () => resolveOutputPath(EToolId.SPAWN_UNPACKER),
  });

  const request: Nullable<ISpawnUnpackRequest> = useMemo(
    () => (source.value && destination.value ? { source: source.value, destination: destination.value } : null),
    [destination.value, source.value]
  );

  const onUnpack = useCallback(
    async (submitted: ISpawnUnpackRequest) => {
      await conversionService.unpack(submitted.source, submitted.destination);
    },
    [conversionService]
  );

  return (
    <JobPickerForm
      operation={conversionService.operation}
      request={request}
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
