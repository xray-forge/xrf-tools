import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import {
  ETextureOpenMode,
  getTextureOpenMode,
  ITextureOpenModeDescriptor,
  TEXTURE_OPEN_MODES,
} from "@/applications/textures-explorer/lib/texture-open-mode";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole } from "@/core/settings/lib/path";
import { EWorkspacePath } from "@/core/settings/lib/workspace-path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { FormRow, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

interface ITexturesExplorerOpenFormProps extends BaseComponentProps {
  /**
   * Called once an open attempt has finished, successfully or not.
   */
  onFinished?: () => void;
}

/**
 * The way into the explorer: browse a game tree, browse a folder of loose textures, or inspect one texture.
 */
export function TexturesExplorerOpenForm({ onFinished }: ITexturesExplorerOpenFormProps): ReactElement {
  const catalogService: TextureCatalogService = useInjection(TextureCatalogService);
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const pathsService: PathsService = useInjection(PathsService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = catalogService.catalog.isLoading || selectionService.selected.isLoading;

  // Browsing is the primary workflow, so it is the default whenever a tree is configured to browse.
  const [modeId, setModeId] = useState<ETextureOpenMode>(
    (pathsService.getPath(EWorkspacePath.GAMEDATA) ?? pathsService.getPath(EWorkspacePath.GAME_INSTALLATION))
      ? ETextureOpenMode.FOLDER
      : ETextureOpenMode.TEXTURE
  );

  const seed = useCallback(() => resolveExistingPathRole(EPathRole.TEXTURES, pathsService.paths), [pathsService.paths]);

  // One picker per mode rather than one that changes shape: each remembers its own path, and a directory picker and a
  // file picker are different dialogs. Called unconditionally, since a hook cannot be chosen the way the rest is.
  const fields: Record<ETextureOpenMode, IPathField> = {
    [ETextureOpenMode.FOLDER]: usePathField({
      application: EApplicationId.TEXTURES_EXPLORER,
      id: "root",
      isDirectory: true,
      isDisabled: isLoading,
      seed,
      title: "Select gamedata or textures directory",
    }),
    [ETextureOpenMode.LOOSE_FOLDER]: usePathField({
      application: EApplicationId.TEXTURES_EXPLORER,
      id: "loose-folder",
      isDirectory: true,
      isDisabled: isLoading,
      seed,
      title: "Select a folder of textures",
    }),
    [ETextureOpenMode.TEXTURE]: usePathField({
      application: EApplicationId.TEXTURES_EXPLORER,
      filters: [{ extensions: ["dds", "thm"], name: "Texture or descriptor" }],
      id: "texture",
      isDisabled: isLoading,
      seed,
      title: "Select dds texture or thm descriptor",
    }),
  };

  const mode: ITextureOpenModeDescriptor = getTextureOpenMode(modeId);
  const field: IPathField = fields[modeId];

  const onOpen = useCallback(async () => {
    if (!field.value) {
      log.info("Cannot open textures without a path");

      return;
    }

    await mode.open(field.value, { catalogService, selectionService });

    onFinished?.();
  }, [catalogService, field.value, log, mode, onFinished, selectionService]);

  return (
    <PickerForm
      isLoading={isLoading}
      title={"Open game textures"}
      description={mode.description}
      error={catalogService.catalog.error?.message ?? selectionService.selected.error?.message}
      submitLabel={mode.submitLabel}
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
          value={modeId}
          disabled={isLoading}
          aria-label={"Open mode"}
          onChange={(_, next: Nullable<ETextureOpenMode>) => next && setModeId(next)}
        >
          {TEXTURE_OPEN_MODES.map((it: ITextureOpenModeDescriptor) => (
            <ToggleButton key={it.id} value={it.id} aria-label={`Open ${it.label.toLowerCase()}`}>
              {it.label}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </FormRow>

      <PathFormRow
        label={mode.field.label}
        description={mode.field.description}
        isDisabled={isLoading}
        field={field}
      />
    </PickerForm>
  );
}
