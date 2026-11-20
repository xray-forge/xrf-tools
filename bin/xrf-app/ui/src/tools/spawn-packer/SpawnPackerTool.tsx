import { flowResult } from "@wirestate/mobx";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo } from "react";

import { ReplaceConfirmDialog } from "@/core/jobs/components/ReplaceConfirmDialog";
import { IReplaceRetry, IReplaceRetryStart, useReplaceRetry } from "@/core/jobs/lib/use-replace-retry";
import { SPAWN_FILE_FILTERS } from "@/core/path/file-filters";
import { EToolId } from "@/core/routing/tool";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { SpawnConversionOutcome } from "@/core/spawn/components/SpawnConversionOutcome";
import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";

interface ISpawnPackRequest {
  /** The directory of unpacked chunks. */
  source: string;
  /** The spawn file written from them. */
  destination: string;
}

/** Build a packed spawn file out of unpacked chunks. */
export function SpawnPackerTool(): ReactElement {
  const conversionService: SpawnConversionService = useInjection(SpawnConversionService);
  const isLoading: boolean = conversionService.operation.isRunning;

  const source: IPathField = usePathField({
    tool: EToolId.SPAWN_PACKER,
    id: "source",
    title: "Select unpacked spawn directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed: () => resolveOutputPath(EToolId.SPAWN_UNPACKER),
  });

  const destination: IPathField = usePathField({
    tool: EToolId.SPAWN_PACKER,
    id: "destination",
    title: "Select spawn file output",
    filters: SPAWN_FILE_FILTERS,
    isSave: true,
    isDisabled: isLoading,
    seed: () => resolveOutputPath(EToolId.SPAWN_PACKER, "all.spawn"),
  });

  const request: Nullable<ISpawnPackRequest> = useMemo(
    () => (source.value && destination.value ? { source: source.value, destination: destination.value } : null),
    [destination.value, source.value]
  );

  const pack: IReplaceRetry<ISpawnPackRequest> = useReplaceRetry(
    useCallback(
      ({ request: submitted, isForced }: IReplaceRetryStart<ISpawnPackRequest>) =>
        flowResult(conversionService.pack({ ...submitted, isForced })),
      [conversionService]
    )
  );

  return (
    <>
      <JobPickerForm
        operation={conversionService.operation}
        request={request}
        isSubmitDisabled={!source.isValid || !destination.isValid}
        title={"Pack spawn file"}
        description={
          "Builds one spawn file from the unpacked chunks; an output file already there is replaced once you agree. " +
          "Cancellation stops before writing; a write already started finishes."
        }
        submitLabel={"Pack"}
        renderResult={(result) => <SpawnConversionOutcome result={result} />}
        onSubmit={pack.run}
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
      </JobPickerForm>

      <ReplaceConfirmDialog retry={pack} />
    </>
  );
}
