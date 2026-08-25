import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { DialogFilter } from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { EApplicationId } from "@/core/routing/application";
import { getExistingProjectLinkedGamePath } from "@/core/settings/lib/path";
import { ProjectService } from "@/core/settings/services/project";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { FormRow } from "@/core/ui/form/FormRow";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";

/** Which of the two things the picker is opening. */
type TOpenMode = "directory" | "archive";

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
  const projectService: ProjectService = useInjection(ProjectService);

  const log: Logger = useLogger("archives");

  const isLoading: boolean = archivesService.project.isLoading;

  // Browsing a directory is the primary workflow, so it is the default whenever there is a project whose archives to
  // browse. Without one, the likelier intent is the isolated volume that was downloaded or extracted on its own.
  const [mode, setMode] = useState<TOpenMode>(projectService.xrfProjectPath ? "directory" : "archive");

  const directory: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_EXPLORER,
    id: "source",
    title: "Select archives directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed: async () =>
      projectService.xrfProjectPath ? getExistingProjectLinkedGamePath(projectService.xrfProjectPath) : null,
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
          ? "Indexes every archive in the directory for browsing. Nothing is written."
          : "Indexes one archive volume for browsing. Nothing is written."
      }
      error={archivesService.project.error ? archivesService.project.error.message : undefined}
      submitLabel={"Open"}
      isSubmitDisabled={!field.isValid}
      onSubmit={onOpen}
    >
      <FormRow label={"Open"} description={"Browse a whole directory, or one archive on its own"} isRequired={false}>
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
