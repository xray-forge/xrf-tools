import { flowResult } from "@wirestate/mobx";
import { useInjection } from "@wirestate/react";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo } from "react";

import { ReplaceConfirmDialog } from "@/core/jobs/components/ReplaceConfirmDialog";
import { IReplaceRetry, IReplaceRetryStart, useReplaceRetry } from "@/core/jobs/lib/use-replace-retry";
import { EToolId } from "@/core/routing/tool";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { ArchivesUnpackerService } from "@/tools/archives-unpacker/services/archives-unpacker.service";

import { ArchivesUnpackResult } from "./components/ArchivesUnpackResult";

interface IUnpackRequest {
  source: string;
  destination: string;
}

export function ArchivesUnpackerTool(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const unpackerService: ArchivesUnpackerService = useInjection(ArchivesUnpackerService);

  const isRunning: boolean = unpackerService.operation.isRunning;

  const source: IPathField = usePathField({
    tool: EToolId.ARCHIVES_UNPACKER,
    id: "source",
    title: "Select archives directory",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const destination: IPathField = usePathField({
    tool: EToolId.ARCHIVES_UNPACKER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
    seed: () => resolveOutputPath(EToolId.ARCHIVES_UNPACKER),
  });

  const request: Nullable<IUnpackRequest> = useMemo(
    () => (source.value && destination.value ? { source: source.value, destination: destination.value } : null),
    [destination.value, source.value]
  );

  const unpack: IReplaceRetry<IUnpackRequest> = useReplaceRetry(
    useCallback(
      ({ request: submitted, isForced }: IReplaceRetryStart<IUnpackRequest>) => {
        log.info("Unpacking:", submitted.source);

        return flowResult(
          unpackerService.unpack({ from: submitted.source, destination: submitted.destination, isForced })
        );
      },
      [log, unpackerService]
    )
  );

  return (
    <>
      <JobPickerForm
        operation={unpackerService.operation}
        request={request}
        isSubmitDisabled={!source.isValid || !destination.isValid}
        title={"Unpack game archives"}
        description={
          "Reads every archive in the source directory and writes its files into the output directory, asking " +
          "before replacing files already there."
        }
        submitLabel={"Unpack"}
        renderResult={(result) => <ArchivesUnpackResult result={result} outputPath={destination.value} />}
        onSubmit={unpack.run}
      >
        <PathFormRow
          isDisabled={isRunning}
          label={"Source"}
          description={"Directory holding the packed game archives"}
          field={source}
        />

        <PathFormRow
          isDisabled={isRunning}
          label={"Output"}
          description={"Directory the archives are unpacked into"}
          field={destination}
        />
      </JobPickerForm>

      <ReplaceConfirmDialog retry={unpack} />
    </>
  );
}
