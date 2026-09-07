import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect } from "react";

import { ArchivesUnpackResult } from "@/applications/archives-unpacker/components/ArchivesUnpackResult";
import { UnpackerService } from "@/applications/archives-unpacker/services/unpacker";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole, resolveOutputPath } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function ArchivesUnpackerApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const pathsService: PathsService = useInjection(PathsService);
  const unpackerService: UnpackerService = useInjection(UnpackerService);

  const job: Nullable<IJobState> = unpackerService.operation.job;
  const isRunning: boolean = unpackerService.operation.isRunning;

  const source: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_UNPACKER,
    id: "source",
    title: "Select archives directory",
    isDirectory: true,
    isDisabled: isRunning,
    seed: () => resolveExistingPathRole(EPathRole.ARCHIVES, pathsService.paths),
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_UNPACKER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
    seed: () => resolveOutputPath(EApplicationId.ARCHIVES_UNPACKER, pathsService.paths),
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

  const onCancel = useCallback(() => unpackerService.operation.cancel(), [unpackerService]);

  // Changing either path invalidates whatever the previous run reported.
  useEffect(() => {
    unpackerService.operation.reset();
  }, [archivesPath, archivesUnpackPath, unpackerService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!source.isValid || !destination.isValid}
      title={"Unpack game archives"}
      description={"Reads every archive in the source directory and writes its files into the output directory."}
      error={unpackerService.operation.error ?? undefined}
      submitLabel={"Unpack"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        unpackerService.operation.result ? (
          <ArchivesUnpackResult result={unpackerService.operation.result} outputPath={archivesUnpackPath} />
        ) : null
      }
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
    </PickerForm>
  );
}
