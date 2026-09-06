import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole } from "@/core/settings/lib/path";
import { EWorkspacePath } from "@/core/settings/lib/workspace-path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { FormRow } from "@/core/ui/form/FormRow";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";

/** Which of the three things the picker is opening. */
type TOpenMode = "folder" | "looseFolder" | "texture";

/** What each mode reads, said before it runs rather than after. */
const MODE_DESCRIPTIONS: Record<TOpenMode, string> = {
  folder:
    "Lists every texture under the root, archives included, and reads what each descriptor declares. Files outside " +
    "the textures directory are counted rather than listed. Nothing is written.",
  looseFolder:
    "Lists every dds under the folder by its own path, for textures that are not in a game tree and have no engine " +
    "reference. Descriptors are not swept, because there are no references to sweep them by. Nothing is written.",
  texture: "Reads one texture and the descriptor beside it. Nothing is written.",
};

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
  const catalogService: TextureCatalogService = useInjection(TextureCatalogService);
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const pathsService: PathsService = useInjection(PathsService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = catalogService.catalog.isLoading || selectionService.selected.isLoading;

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

  const looseFolder: IPathField = usePathField({
    application: EApplicationId.TEXTURES_EXPLORER,
    id: "loose-folder",
    title: "Select a folder of textures",
    isDirectory: true,
    isDisabled: isLoading,
    seed,
  });

  const field: IPathField = { folder: root, looseFolder, texture }[mode];

  const onOpen = useCallback(async () => {
    if (!field.value) {
      log.info("Cannot open textures without a path");

      return;
    }

    // Either mode starts a session rather than adding to one, so whatever the other mode had open is closed first: a
    // texture from a previous root has nothing to do with the roots being opened now.
    if (mode === "folder") {
      await catalogService.openRoot(field.value);
    } else if (mode === "looseFolder") {
      await catalogService.openLooseDirectory(field.value);
    } else {
      await catalogService.close();
      await selectionService.openFile(field.value);
    }

    onFinished?.();
  }, [catalogService, field.value, log, mode, onFinished, selectionService]);

  return (
    <PickerForm
      isLoading={isLoading}
      title={"Open game textures"}
      description={MODE_DESCRIPTIONS[mode]}
      error={catalogService.catalog.error?.message ?? selectionService.selected.error?.message}
      submitLabel={mode === "texture" ? "Open" : "Browse"}
      isSubmitDisabled={!field.isValid}
      onSubmit={onOpen}
    >
      <FormRow
        label={"Open"}
        description={"A game tree, a folder of loose textures, or one texture on its own"}
        isRequired={false}
      >
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
          <ToggleButton value={"looseFolder"} aria-label={"Open loose folder"}>
            Loose folder
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
      ) : null}

      {mode === "looseFolder" ? (
        <PathFormRow
          label={"Textures folder"}
          description={"Any directory holding dds files, in a game tree or not"}
          isDisabled={isLoading}
          field={looseFolder}
        />
      ) : null}

      {mode === "texture" ? (
        <PathFormRow
          label={"Texture file"}
          description={"Dds texture, or the thm descriptor beside it"}
          isDisabled={isLoading}
          field={texture}
        />
      ) : null}
    </PickerForm>
  );
}
