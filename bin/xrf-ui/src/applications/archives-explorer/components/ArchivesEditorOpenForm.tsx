import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import {
  ARCHIVE_FILTERS,
  EArchiveOpenMode,
  OPEN_MODE_OPTIONS,
  OPEN_MODES,
} from "@/applications/archives-explorer/components/ArchivesEditorOpenForm.utils";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ChoiceFormRow, IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";

/**
 * The way into the explorer: index a directory of volumes, or one volume on its own.
 */
export function ArchivesEditorOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const archivesService: ArchivesService = useInjection(ArchivesService);

  const isLoading: boolean = archivesService.project.isLoading;

  // Browsing a directory is the primary workflow, so it is the fallback - and after that, whichever of the two was
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

  const field: IPathField = mode === EArchiveOpenMode.DIRECTORY ? directory : archive;

  const onOpen = useCallback(() => {
    if (field.value) {
      archivesService.openProject(field.value);
    } else {
      log.info("Cannot parse archives project without path");
    }
  }, [archivesService, field.value, log]);

  return (
    <PickerForm
      isLoading={isLoading}
      title={"Open game archives"}
      description={
        mode === EArchiveOpenMode.DIRECTORY
          ? "Indexes every archive in the directory for browsing."
          : "Indexes one archive volume for browsing."
      }
      error={archivesService.project.error ? archivesService.project.error.message : undefined}
      submitLabel={"Open"}
      isSubmitDisabled={!field.isValid}
      onSubmit={onOpen}
    >
      <ChoiceFormRow
        label={"Open"}
        description={"Browse a whole directory, or one archive on its own"}
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
      ) : (
        <PathFormRow
          isDisabled={isLoading}
          label={"Archive volume"}
          description={"Single packed archive to open"}
          field={archive}
        />
      )}
    </PickerForm>
  );
}
