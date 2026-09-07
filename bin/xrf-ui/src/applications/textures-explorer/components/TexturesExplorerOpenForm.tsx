import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import {
  ETextureOpenMode,
  getTextureOpenMode,
  ITextureOpenModeDescriptor,
  TEXTURE_OPEN_MODE_IDS,
  TEXTURE_OPEN_MODES,
} from "@/applications/textures-explorer/lib/texture-open-mode";
import { AssetRootFormRow } from "@/core/assets/components/AssetRootFormRow";
import { useAssetRootField } from "@/core/assets/lib";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { FormRow, IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
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

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = catalogService.catalog.isLoading || selectionService.selected.isLoading;

  // Browsing is the primary workflow, so it is the fallback - but what someone last opened is a better guess, and
  // someone authoring single textures should not re-pick the mode every session.
  const [modeId, setModeId] = useRememberedValue<ETextureOpenMode>({
    allowed: TEXTURE_OPEN_MODE_IDS,
    application: EApplicationId.TEXTURES_EXPLORER,
    fallback: ETextureOpenMode.FOLDER,
    id: "mode",
  });

  // One picker per mode rather than one that changes shape: each remembers its own path, and a directory picker and a
  // file picker are different dialogs. Called unconditionally, since a hook cannot be chosen the way the rest is.
  const fields: Record<ETextureOpenMode, IPathField> = {
    [ETextureOpenMode.FOLDER]: usePathField({
      application: EApplicationId.TEXTURES_EXPLORER,
      id: "root",
      isDirectory: true,
      isDisabled: isLoading,
      title: "Select gamedata or textures directory",
    }),
    [ETextureOpenMode.LOOSE_FOLDER]: usePathField({
      application: EApplicationId.TEXTURES_EXPLORER,
      id: "loose-folder",
      isDirectory: true,
      isDisabled: isLoading,
      title: "Select a folder of textures",
    }),
    [ETextureOpenMode.TEXTURE]: usePathField({
      application: EApplicationId.TEXTURES_EXPLORER,
      filters: [{ extensions: ["dds", "thm"], name: "Texture or descriptor" }],
      id: "texture",
      isDisabled: isLoading,
      title: "Select dds texture or thm descriptor",
    }),
  };

  const assetRoot: IPathField = useAssetRootField(EApplicationId.TEXTURES_EXPLORER, isLoading);

  const mode: ITextureOpenModeDescriptor = getTextureOpenMode(modeId);
  const field: IPathField = fields[modeId];

  const onOpen = useCallback(async () => {
    if (!field.value) {
      log.info("Cannot open textures without a path");

      return;
    }

    await mode.open(field.value, { assetRoot: assetRoot.value, catalogService, selectionService });

    onFinished?.();
  }, [assetRoot.value, catalogService, field.value, log, mode, onFinished, selectionService]);

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

      <PathFormRow label={mode.field.label} description={mode.field.description} isDisabled={isLoading} field={field} />

      <AssetRootFormRow field={assetRoot} isDisabled={isLoading} />
    </PickerForm>
  );
}
