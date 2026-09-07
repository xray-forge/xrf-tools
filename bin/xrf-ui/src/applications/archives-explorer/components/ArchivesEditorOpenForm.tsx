import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { DialogFilter } from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { FormRow, IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";

/** Which of the two things the picker is opening. */
type TOpenMode = "directory" | "archive";

const OPEN_MODES: ReadonlyArray<TOpenMode> = ["directory", "archive"];

/** Volume extensions offered by the dialog. */
const ARCHIVE_FILTERS: Array<DialogFilter> = [
  {
    name: "Archive volume",
    extensions: ["db", "xdb"].flatMap((base: string) => [
      base,
      ...Array.from({ length: 10 }, (_, index: number) => `${base}${index}`),
    ]),
  },
  { name: "All files", extensions: ["*"] },
];

/**
 * The way into the explorer: index a directory of volumes, or one volume on its own.
 */
export function ArchivesEditorOpenForm(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = archivesService.project.isLoading;

  // Browsing a directory is the primary workflow, so it is the fallback - and after that, whichever of the two was
  // last used, because someone who opens single volumes they downloaded does so every time.
  const [mode, setMode] = useRememberedValue<TOpenMode>({
    allowed: OPEN_MODES,
    application: EApplicationId.ARCHIVES_EXPLORER,
    fallback: "directory",
    id: "mode",
  });

  const directory: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_EXPLORER,
    id: "source",
    title: "Select archives directory",
    isDirectory: true,
    isDisabled: isLoading,
  });

  // Unseeded on purpose: the only path a project offers is a directory, which would sit in a volume field looking like
  // a choice without even giving the dialog somewhere to start.
  const archive: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_EXPLORER,
    id: "archive",
    title: "Select archive volume",
    filters: ARCHIVE_FILTERS,
    isDisabled: isLoading,
  });

  const field: IPathField = mode === "directory" ? directory : archive;

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
        mode === "directory"
          ? "Indexes every archive in the directory for browsing."
          : "Indexes one archive volume for browsing."
      }
      error={archivesService.project.error ? archivesService.project.error.message : undefined}
      submitLabel={"Open"}
      isSubmitDisabled={!field.isValid}
      onSubmit={onOpen}
    >
      <FormRow label={"Open"} description={"Browse a whole directory, or one archive on its own"}>
        <ToggleButtonGroup
          aria-label={"Open mode"}
          exclusive={true}
          size={"small"}
          value={mode}
          disabled={isLoading}
          onChange={(_, next: TOpenMode) => next && setMode(next)}
        >
          <ToggleButton value={"directory"} aria-label={"Open directory"}>
            Directory
          </ToggleButton>
          <ToggleButton value={"archive"} aria-label={"Open archive"}>
            Archive
          </ToggleButton>
        </ToggleButtonGroup>
      </FormRow>

      {mode === "directory" ? (
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
