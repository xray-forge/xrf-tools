import { Alert } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { spawnCommands } from "@/core/bindings/commands/spawn";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { resolveOutputPath } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Build a packed spawn file from chunks on disk.
 */
export function SpawnEditorPackForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);
  const notify: TEmitNotification = useEmitNotification();

  const pathsService: PathsService = useInjection(PathsService);

  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<Nullable<string>>(null);
  const [packedTo, setPackedTo] = useState<Nullable<string>>(null);

  const source: IPathField = usePathField({
    application: EApplicationId.SPAWN_PACKER,
    id: "source",
    title: "Select unpacked spawn directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed: () => resolveOutputPath(EApplicationId.SPAWN_UNPACKER, pathsService.paths),
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.SPAWN_PACKER,
    id: "destination",
    title: "Select spawn file output",
    filters: [{ name: "spawn", extensions: ["spawn"] }],
    isSave: true,
    isDisabled: isLoading,
    seed: () => resolveOutputPath(EApplicationId.SPAWN_PACKER, pathsService.paths, "all.spawn"),
  });

  const onPack = useCallback(async () => {
    if (!source.value || !destination.value) {
      return log.error("Cannot pack spawn file, expected correct paths");
    }

    log.info("Packing spawn file:", source.value, destination.value);

    setIsLoading(true);
    setError(null);
    setPackedTo(null);

    try {
      await spawnCommands.packFile(source.value, destination.value);

      setPackedTo(destination.value);

      notify({
        details: `${source.value}\n${destination.value}`,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.SPAWN_PACKER,
        title: "Packed spawn file",
      });
    } catch (caught: unknown) {
      log.error("Failed to pack spawn file:", caught);
      setError(String(caught));

      notify({
        details: `${source.value}\n${String(caught)}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.SPAWN_PACKER,
        title: "Could not pack spawn file",
      });
    } finally {
      setIsLoading(false);
    }
  }, [destination.value, log, notify, source.value]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!source.isValid || !destination.isValid}
      title={"Pack spawn file"}
      description={"Builds one spawn file from the unpacked chunks. The output file is overwritten."}
      error={error ?? undefined}
      submitLabel={"Pack"}
      status={
        packedTo ? (
          <Alert severity={"success"} variant={"outlined"}>
            Successfully packed spawn to {packedTo}
          </Alert>
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
