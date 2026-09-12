import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { AssetRootFormRow } from "@/core/assets/components/AssetRootFormRow";
import { useAssetRootField } from "@/core/assets/lib";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ChoiceFormRow, IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";

import { EVisualOpenMode, OPEN_MODE_OPTIONS, OPEN_MODES } from "./VisualsExplorerOpenForm.utils";

interface IVisualsExplorerOpenFormProps extends BaseComponentProps {
  /**
   * Called once an open attempt has finished, successfully or not.
   */
  onFinished?: () => void;
}

/**
 * The way into the explorer: browse a root, or look at one model.
 *
 * One row whose dialog follows the mode rather than two rows and a rule for when both are filled. Each mode keeps its
 * own remembered path, so switching back does not cost the last folder or the last file.
 */
export function VisualsExplorerOpenForm({
  "data-testid": dataTestId = "visuals-explorer-open-form",
  id,
  className,
  onFinished,
}: IVisualsExplorerOpenFormProps): ReactElement {
  const visualsService: VisualsService = useInjection(VisualsService);
  const browseService: VisualsBrowseService = useInjection(VisualsBrowseService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = visualsService.visual.isLoading || browseService.visuals.isLoading;

  // Browsing is the primary workflow, so it is the fallback - but what someone last opened is a better guess than
  // that, and a person who only ever looks at single models should not re-pick the mode every session.
  const [mode, setMode] = useRememberedValue<EVisualOpenMode>({
    allowed: OPEN_MODES,
    application: EApplicationId.VISUALS_EXPLORER,
    fallback: EVisualOpenMode.FOLDER,
    id: "mode",
  });

  const visual: IPathField = usePathField({
    application: EApplicationId.VISUALS_EXPLORER,
    id: "visual",
    title: "Select ogf visual",
    filters: [{ name: "Ogf visual", extensions: ["ogf"] }],
    isDisabled: isLoading,
  });

  const root: IPathField = usePathField({
    application: EApplicationId.VISUALS_EXPLORER,
    id: "root",
    title: "Select gamedata or meshes directory",
    isDirectory: true,
    isDisabled: isLoading,
  });

  const assetRoot: IPathField = useAssetRootField(EApplicationId.VISUALS_EXPLORER, isLoading);

  const field: IPathField = mode === EVisualOpenMode.FOLDER ? root : visual;

  const onOpen = useCallback(async () => {
    if (!field.value) {
      log.info("Cannot open a visual without a path");

      return;
    }

    // Either mode starts a session rather than adding to one, so whatever the other mode had open is closed first: a
    // model from a previous root has nothing to do with the roots being opened now, and leaving it on screen beside a
    // tree that does not contain it is the kind of disagreement the viewport is supposed to prevent.
    if (mode === EVisualOpenMode.FOLDER) {
      await visualsService.close();
      await browseService.openRoot(field.value, assetRoot.value);
    } else {
      await browseService.close();
      await visualsService.openFile(field.value, assetRoot.value);
    }

    onFinished?.();
  }, [assetRoot.value, browseService, field.value, log, mode, onFinished, visualsService]);

  return (
    <PickerForm
      data-testid={dataTestId}
      id={id}
      className={className}
      isLoading={isLoading}
      title={"Open game visuals"}
      description={
        mode === EVisualOpenMode.FOLDER
          ? "Lists every visual under the root, archives included. Nothing is written."
          : "Reads the model and shows its bind pose. Nothing is written."
      }
      error={visualsService.visual.error?.message ?? browseService.visuals.error?.message}
      submitLabel={mode === EVisualOpenMode.FOLDER ? "Browse" : "Open"}
      isSubmitDisabled={!field.isValid}
      onSubmit={onOpen}
    >
      <ChoiceFormRow
        label={"Open"}
        description={"Browse a whole root, or one model on its own"}
        options={OPEN_MODE_OPTIONS}
        value={mode}
        isRequired={false}
        isDisabled={isLoading}
        onChange={setMode}
      />

      {mode === EVisualOpenMode.FOLDER ? (
        <PathFormRow
          label={"Meshes root"}
          description={"Gamedata directory to browse"}
          isDisabled={isLoading}
          field={root}
        />
      ) : (
        <PathFormRow label={"Visual file"} description={"Ogf model to preview"} isDisabled={isLoading} field={visual} />
      )}

      <AssetRootFormRow field={assetRoot} isDisabled={isLoading} />
    </PickerForm>
  );
}
