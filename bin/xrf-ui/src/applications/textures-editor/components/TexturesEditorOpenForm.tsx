import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
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
  const pathsService: PathsService = useInjection(PathsService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = selectionService.selected.isLoading;

  const texture: IPathField = usePathField({
    application: EApplicationId.TEXTURES_EDITOR,
    id: "texture",
    title: "Select dds texture or thm descriptor",
    filters: [{ name: "Texture or descriptor", extensions: ["dds", "thm"] }],
    isDisabled: isLoading,
    seed: useCallback(() => resolveExistingPathRole(EPathRole.TEXTURES, pathsService.paths), [pathsService.paths]),
  });

  const onOpen = useCallback(async () => {
    if (!texture.value) {
      log.info("Cannot open a texture without a path");

      return;
    }

    await selectionService.openFile(texture.value);
  }, [log, selectionService, texture.value]);

  return (
    <PickerForm
      data-testid={dataTestId}
      id={id}
      className={className}
      isLoading={isLoading}
      title={"Open a texture to work on"}
      description={
        "Opens one texture and the descriptor beside it, authoring a descriptor where there is none. The bump pair " +
        "and detail texture it names are resolved through the configured game data."
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
    </PickerForm>
  );
}
