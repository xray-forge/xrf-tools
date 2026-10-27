import { useInjection } from "@wirestate/react";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { ReactElement, useCallback } from "react";

import { VisualsSequencerService } from "@/applications/visuals-sequencer/services/visuals-sequencer";
import { AssetRootFormRow } from "@/core/assets/components/AssetRootFormRow";
import { useAssetRootField } from "@/core/assets/lib";
import { OGF_FILE_FILTERS } from "@/core/path/file-filters";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IVisualsSequencerOpenFormProps extends BaseComponentProps {
  /** Called once an open attempt has finished, successfully or not. */
  onFinished?: () => void;
}

/**
 * The way into the sequencer: one model, whose motions a track is written out of.
 */
export function VisualsSequencerOpenForm({ onFinished }: IVisualsSequencerOpenFormProps): ReactElement {
  const sequencerService: VisualsSequencerService = useInjection(VisualsSequencerService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = sequencerService.visual.isLoading;

  const visual: IPathField = usePathField({
    application: EApplicationId.VISUALS_SEQUENCER,
    id: "visual",
    title: "Select ogf visual",
    filters: OGF_FILE_FILTERS,
    isDisabled: isLoading,
  });

  const assetRoot: IPathField = useAssetRootField(EApplicationId.VISUALS_SEQUENCER, isLoading);

  const onOpen = useCallback(async () => {
    if (!visual.value) {
      log.info("Cannot open a visual without a path");

      return;
    }

    await sequencerService.openFile(visual.value, assetRoot.value);

    onFinished?.();
  }, [assetRoot.value, log, onFinished, sequencerService, visual.value]);

  return (
    <PickerForm
      isLoading={isLoading}
      title={"Open a visual to sequence"}
      description={"Reads the model and names every motion it can play. Nothing is written."}
      error={sequencerService.visual.error?.message}
      submitLabel={"Open"}
      isSubmitDisabled={!visual.isValid}
      onSubmit={onOpen}
    >
      <PathFormRow
        label={"Visual file"}
        description={"Ogf model whose motions the track is built from"}
        isDisabled={isLoading}
        field={visual}
      />

      <AssetRootFormRow field={assetRoot} isDisabled={isLoading} />
    </PickerForm>
  );
}
