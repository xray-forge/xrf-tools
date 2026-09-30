import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect } from "react";

import { UnpackerService } from "@/applications/archives-unpacker/services/unpacker";
import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { EApplicationId } from "@/core/routing/application";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";

import { ArchivesUnpackResult } from "./components/ArchivesUnpackResult";

export function ArchivesUnpackerApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const unpackerService: UnpackerService = useInjection(UnpackerService);

  const isRunning: boolean = unpackerService.operation.isRunning;

  const source: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_UNPACKER,
    id: "source",
    title: "Select archives directory",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_UNPACKER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
    seed: () => resolveOutputPath(EApplicationId.ARCHIVES_UNPACKER),
  });

  const archivesPath: Nullable<string> = source.value;
  const archivesUnpackPath: Nullable<string> = destination.value;

  const onUnpackArchivesPathClicked = useCallback(async () => {
    if (!archivesPath || !archivesUnpackPath) {
      return;
    }

    log.info("Unpacking:", archivesPath);

    await unpackerService.unpack(archivesPath, archivesUnpackPath);
  }, [archivesPath, archivesUnpackPath, log, unpackerService]);

  // Changing either path invalidates whatever the previous run reported.
  useEffect(() => {
    unpackerService.operation.reset();
  }, [archivesPath, archivesUnpackPath, unpackerService]);

  return (
    <JobPickerForm
      operation={unpackerService.operation}
      isSubmitDisabled={!source.isValid || !destination.isValid}
      title={"Unpack game archives"}
      description={"Reads every archive in the source directory and writes its files into the output directory."}
      submitLabel={"Unpack"}
      renderResult={(result) => <ArchivesUnpackResult result={result} outputPath={archivesUnpackPath} />}
      onSubmit={onUnpackArchivesPathClicked}
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
  );
}
