import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole } from "@/core/settings/lib/path";
import { EWorkspacePath } from "@/core/settings/lib/workspace-path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { FormRow } from "@/core/ui/form/FormRow";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";

/** Which of the two things the picker is opening. */
type TOpenMode = "folder" | "model";

interface IVisualsExplorerOpenFormProps extends BaseComponentProps {
  /**
   * Called once an open attempt has finished, successfully or not.
   *
   * A failed open leaves nothing on screen, so the form stays visible with its error either way; this
   * only dismisses a picker that was reopened over a model.
   */
  onFinished?: () => void;
}

/**
 * The way into the explorer: browse a root, or look at one model.
 *
 * One row whose dialog follows the mode rather than two rows and a rule for when both are filled. Each mode keeps its
 * own remembered path, so switching back does not cost the last folder or the last file.
 */
export function VisualsExplorerOpenForm({ onFinished }: IVisualsExplorerOpenFormProps): ReactElement {
  const visualsService: VisualsService = useInjection(VisualsService);
  const browseService: VisualsBrowseService = useInjection(VisualsBrowseService);
  const pathsService: PathsService = useInjection(PathsService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = visualsService.visual.isLoading || browseService.visuals.isLoading;

  // Browsing is the primary workflow, so it is the default whenever a tree is configured to browse.
  const [mode, setMode] = useState<TOpenMode>(
    pathsService.getPath(EWorkspacePath.GAMEDATA) ?? pathsService.getPath(EWorkspacePath.GAME_INSTALLATION)
      ? "folder"
      : "model"
  );

  const seed = useCallback(
    () => resolveExistingPathRole(EPathRole.VISUALS, pathsService.paths),
    [pathsService.paths]
  );

  const visual: IPathField = usePathField({
    application: EApplicationId.VISUALS_EXPLORER,
    id: "visual",
    title: "Select ogf visual",
    filters: [{ name: "Ogf visual", extensions: ["ogf"] }],
    isDisabled: isLoading,
    seed,
  });

  const root: IPathField = usePathField({
    application: EApplicationId.VISUALS_EXPLORER,
    id: "root",
    title: "Select gamedata or meshes directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed,
  });

  const field: IPathField = mode === "folder" ? root : visual;

  const onOpen = useCallback(async () => {
    if (!field.value) {
      log.info("Cannot open a visual without a path");

      return;
    }

    // Either mode starts a session rather than adding to one, so whatever the other mode had open is closed first: a
    // model from a previous root has nothing to do with the roots being opened now, and leaving it on screen beside a
    // tree that does not contain it is the kind of disagreement the viewport is supposed to prevent.
    if (mode === "folder") {
      await visualsService.close();
      await browseService.openRoot(field.value);
    } else {
      await browseService.close();
      await visualsService.openFile(field.value);
    }

    onFinished?.();
  }, [browseService, field.value, log, mode, onFinished, visualsService]);

  return (
    <PickerForm
      isLoading={isLoading}
      title={"Open game visuals"}
      description={
        mode === "folder"
          ? "Lists every visual under the root, archives included. Nothing is written."
          : "Reads the model and shows its bind pose. Nothing is written."
      }
      error={visualsService.visual.error?.message ?? browseService.visuals.error?.message}
      submitLabel={mode === "folder" ? "Browse" : "Open"}
      isSubmitDisabled={!field.isValid}
      onSubmit={onOpen}
    >
      <FormRow label={"Open"} description={"Browse a whole root, or one model on its own"} isRequired={false}>
        <ToggleButtonGroup
          exclusive
          size={"small"}
          value={mode}
          disabled={isLoading}
          aria-label={"Open mode"}
          onChange={(_, next: TOpenMode) => next && setMode(next)}
        >
          <ToggleButton value={"folder"} aria-label={"Open folder"}>
            Folder
          </ToggleButton>
          <ToggleButton value={"model"} aria-label={"Open model"}>
            Model
          </ToggleButton>
        </ToggleButtonGroup>
      </FormRow>

      {mode === "folder" ? (
        <PathFormRow
          label={"Meshes root"}
          description={"Gamedata directory to browse"}
          isDisabled={isLoading}
          field={root}
        />
      ) : (
        <PathFormRow label={"Visual file"} description={"Ogf model to preview"} isDisabled={isLoading} field={visual} />
      )}
    </PickerForm>
  );
}
