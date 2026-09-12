import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import {
  ARCHIVE_FILTERS,
  EArchiveOpenMode,
  OPEN_MODE_DESCRIPTIONS,
  OPEN_MODE_OPTIONS,
  OPEN_MODES,
} from "@/applications/archives-explorer/components/ArchivesEditorOpenForm.utils";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { createRoot } from "@/core/assets/lib";
import { useRootProbe } from "@/core/assets/lib/use-root-probe";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ChoiceFormRow, IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * The way into the explorer: index a directory of volumes, one volume on its own, or a whole game folder.
 */
export function ArchivesEditorOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const archivesService: ArchivesService = useInjection(ArchivesService);

  const isLoading: boolean = archivesService.subject.isLoading;

  // Browsing a directory is the primary workflow, so it is the fallback - and after that, whichever of the three was
  // last used, because someone who opens single volumes they downloaded does so every time.
  const [mode, setMode] = useRememberedValue<EArchiveOpenMode>({
    id: "mode",
    application: EApplicationId.ARCHIVES_EXPLORER,
    allowed: OPEN_MODES,
    fallback: EArchiveOpenMode.DIRECTORY,
  });

  const directory: IPathField = usePathField({
    id: "source",
    application: EApplicationId.ARCHIVES_EXPLORER,
    title: "Select archives directory",
    isDirectory: true,
    isDisabled: isLoading,
  });

  const archive: IPathField = usePathField({
    id: "archive",
    application: EApplicationId.ARCHIVES_EXPLORER,
    title: "Select archive volume",
    filters: ARCHIVE_FILTERS,
    isDisabled: isLoading,
  });

  const installation: IPathField = usePathField({
    id: "installation",
    application: EApplicationId.ARCHIVES_EXPLORER,
    title: "Select game folder",
    isDirectory: true,
    isDisabled: isLoading,
  });

  // A folder named by hand is a guess until something confirms it, and this is the mode where guessing wrong is
  // quiet: a game folder named one level too high mounts nothing and reads as an installation with no files.
  const installationFact: Nullable<string> = useRootProbe(
    mode === EArchiveOpenMode.INSTALLATION && !installation.error ? installation.value : null
  );

  const field: IPathField =
    mode === EArchiveOpenMode.DIRECTORY ? directory : mode === EArchiveOpenMode.ARCHIVE ? archive : installation;

  const onOpen = useCallback(() => {
    if (!field.value) {
      log.info("Cannot parse archives project without path");

      return;
    }

    if (mode === EArchiveOpenMode.INSTALLATION) {
      archivesService.openWorld({ asset: null, roots: [createRoot(field.value)] });
    } else {
      archivesService.openVolumes(field.value);
    }
  }, [archivesService, field.value, log, mode]);

  return (
    <PickerForm
      isLoading={isLoading}
      title={"Open game archives"}
      description={OPEN_MODE_DESCRIPTIONS[mode]}
      error={archivesService.subject.error ? archivesService.subject.error.message : undefined}
      submitLabel={"Open"}
      isSubmitDisabled={!field.isValid}
      onSubmit={onOpen}
    >
      <ChoiceFormRow
        label={"Open"}
        description={"Browse a whole directory, one archive on its own, or the game as the engine mounts it"}
        options={OPEN_MODE_OPTIONS}
        value={mode}
        isDisabled={isLoading}
        onChange={setMode}
      />

      {mode === EArchiveOpenMode.DIRECTORY ? (
        <PathFormRow
          isDisabled={isLoading}
          label={"Archives directory"}
          description={"Directory holding the packed game archives"}
          field={directory}
        />
      ) : mode === EArchiveOpenMode.ARCHIVE ? (
        <PathFormRow
          isDisabled={isLoading}
          label={"Archive volume"}
          description={"Single packed archive to open"}
          field={archive}
        />
      ) : (
        <PathFormRow
          isDisabled={isLoading}
          label={"Game folder"}
          description={"Folder holding fsgame.ltx, or a game data tree on its own"}
          fact={installationFact}
          field={installation}
        />
      )}
    </PickerForm>
  );
}
