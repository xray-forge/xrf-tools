import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { ArchivesUnpackResult } from "@/applications/archives-unpacker/components/ArchivesUnpackResult";
import { archivesCommands } from "@/core/bindings/commands/archives";
import { ArchiveUnpackResult } from "@/core/bindings/types/xrf-pack";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole, resolveOutputPath } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function ArchivesUnpackerApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);
  const notify: TEmitNotification = useEmitNotification();

  const pathsService: PathsService = useInjection(PathsService);

  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<Nullable<string>>(null);
  const [result, setResult] = useState<Nullable<ArchiveUnpackResult>>(null);

  const source: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_UNPACKER,
    id: "source",
    title: "Select archives directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed: () => resolveExistingPathRole(EPathRole.ARCHIVES, pathsService.paths),
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_UNPACKER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isLoading,
    seed: () => resolveOutputPath(EApplicationId.ARCHIVES_UNPACKER, pathsService.paths),
  });

  const archivesPath: Nullable<string> = source.value;
  const archivesUnpackPath: Nullable<string> = destination.value;

  const onUnpackArchivesPathClicked = useCallback(async () => {
    if (!archivesPath || !archivesUnpackPath) {
      return;
    }

    try {
      setIsLoading(true);
      setResult(null);
      setError(null);

      log.info("Unpacking:", archivesPath);

      const result: ArchiveUnpackResult = await archivesCommands.unpackDirectory(archivesPath, archivesUnpackPath);

      log.info("Unpacked:", archivesPath);

      setResult(result);

      notify({
        details: `${archivesPath}\n${archivesUnpackPath}`,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.ARCHIVES_UNPACKER,
        title: "Unpacked archives",
      });
    } catch (error: unknown) {
      log.error("Unpack error:", error);
      setError(String(error));

      notify({
        details: `${archivesPath}\n${String(error)}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.ARCHIVES_UNPACKER,
        title: "Could not unpack archives",
      });
    } finally {
      setIsLoading(false);
    }
  }, [archivesPath, archivesUnpackPath, log, notify]);

  // Changing either path invalidates whatever the previous run reported.
  useEffect(() => {
    setError(null);
    setResult(null);
  }, [archivesPath, archivesUnpackPath]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!source.isValid || !destination.isValid}
      title={"Unpack game archives"}
      description={"Reads every archive in the source directory and writes its files into the output directory."}
      error={error ?? undefined}
      submitLabel={"Unpack"}
      result={result ? <ArchivesUnpackResult result={result} outputPath={archivesUnpackPath} /> : null}
      onSubmit={onUnpackArchivesPathClicked}
    >
      <PathFormRow
        isDisabled={isLoading}
        label={"Source"}
        description={"Directory holding the packed game archives"}
        field={source}
      />

      <PathFormRow
        isDisabled={isLoading}
        label={"Output"}
        description={"Directory the archives are unpacked into"}
        field={destination}
      />
    </PickerForm>
  );
}
