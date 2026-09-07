import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { AssetRootFormRow } from "@/core/assets/components/AssetRootFormRow";
import { useAssetRootField } from "@/core/assets/lib";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";

/**
 * The way into the editor: one texture.
 */
export function TexturesEditorOpenForm({
  "data-testid": dataTestId = "textures-editor-open-form",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = selectionService.selected.isLoading;

  const texture: IPathField = usePathField({
    application: EApplicationId.TEXTURES_EDITOR,
    id: "texture",
    title: "Select dds texture or thm descriptor",
    filters: [{ name: "Texture or descriptor", extensions: ["dds", "thm"] }],
    isDisabled: isLoading,
  });

  const assetRoot: IPathField = useAssetRootField(EApplicationId.TEXTURES_EDITOR, isLoading);

  const onOpen = useCallback(async () => {
    if (!texture.value) {
      log.info("Cannot open a texture without a path");

      return;
    }

    selectionService.setAssetRoot(assetRoot.value);

    await selectionService.openFile(texture.value);
  }, [assetRoot.value, log, selectionService, texture.value]);

  return (
    <PickerForm
      data-testid={dataTestId}
      id={id}
      className={className}
      isLoading={isLoading}
      title={"Open a texture to work on"}
      description={
        "Opens one texture and the descriptor beside it, authoring a descriptor where there is none. The bump pair " +
        "and detail texture it names are resolved in its own tree, and in whatever further tree is named below."
      }
      error={selectionService.selected.error?.message}
      submitLabel={"Open"}
      isSubmitDisabled={!texture.isValid}
      onSubmit={onOpen}
    >
      <PathFormRow
        label={"Texture file"}
        description={"Dds texture, or the thm descriptor beside it"}
        isDisabled={isLoading}
        field={texture}
      />

      <AssetRootFormRow field={assetRoot} isDisabled={isLoading} />
    </PickerForm>
  );
}
