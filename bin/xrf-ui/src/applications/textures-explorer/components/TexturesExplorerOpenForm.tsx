import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { TexturesService } from "@/applications/textures-explorer/services/textures";
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
type TOpenMode = "folder" | "texture";

interface ITexturesExplorerOpenFormProps extends BaseComponentProps {
  /**
   * Called once an open attempt has finished, successfully or not.
   */
  onFinished?: () => void;
}

/**
 * The way into the explorer: browse a root, or inspect one texture.
 */
export function TexturesExplorerOpenForm({ onFinished }: ITexturesExplorerOpenFormProps): ReactElement {
  const texturesService: TexturesService = useInjection(TexturesService);
  const pathsService: PathsService = useInjection(PathsService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = texturesService.catalog.isLoading || texturesService.selected.isLoading;

  // Browsing is the primary workflow, so it is the default whenever a tree is configured to browse.
  const [mode, setMode] = useState<TOpenMode>(
    (pathsService.getPath(EWorkspacePath.GAMEDATA) ?? pathsService.getPath(EWorkspacePath.GAME_INSTALLATION))
      ? "folder"
      : "texture"
  );

  const seed = useCallback(() => resolveExistingPathRole(EPathRole.TEXTURES, pathsService.paths), [pathsService.paths]);

  const texture: IPathField = usePathField({
    application: EApplicationId.TEXTURES_EXPLORER,
    id: "texture",
    title: "Select dds texture or thm descriptor",
    filters: [{ name: "Texture or descriptor", extensions: ["dds", "thm"] }],
    isDisabled: isLoading,
    seed,
  });

  const root: IPathField = usePathField({
    application: EApplicationId.TEXTURES_EXPLORER,
    id: "root",
    title: "Select gamedata or textures directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed,
  });

  const field: IPathField = mode === "folder" ? root : texture;

  const onOpen = useCallback(async () => {
    if (!field.value) {
      log.info("Cannot open textures without a path");

      return;
    }

    // Either mode starts a session rather than adding to one, so whatever the other mode had open is closed first: a
    // texture from a previous root has nothing to do with the roots being opened now.
    if (mode === "folder") {
      await texturesService.openRoot(field.value);
    } else {
      await texturesService.close();
      await texturesService.openFile(field.value);
    }

    onFinished?.();
  }, [field.value, log, mode, onFinished, texturesService]);

  return (
    <PickerForm
      isLoading={isLoading}
      title={"Open game textures"}
      description={
        mode === "folder"
          ? "Lists every texture under the root, archives included, and reads what each descriptor declares. Nothing " +
            "is written."
          : "Reads one texture and the descriptor beside it. Nothing is written."
      }
      error={texturesService.catalog.error?.message ?? texturesService.selected.error?.message}
      submitLabel={mode === "folder" ? "Browse" : "Open"}
      isSubmitDisabled={!field.isValid}
      onSubmit={onOpen}
    >
      <FormRow label={"Open"} description={"Browse a whole root, or one texture on its own"} isRequired={false}>
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
          <ToggleButton value={"texture"} aria-label={"Open texture"}>
            Texture
          </ToggleButton>
        </ToggleButtonGroup>
      </FormRow>

      {mode === "folder" ? (
        <PathFormRow
          label={"Textures root"}
          description={"Gamedata directory to browse"}
          isDisabled={isLoading}
          field={root}
        />
      ) : (
        <PathFormRow
          label={"Texture file"}
          description={"Dds texture, or the thm descriptor beside it"}
          isDisabled={isLoading}
          field={texture}
        />
      )}
    </PickerForm>
  );
}
