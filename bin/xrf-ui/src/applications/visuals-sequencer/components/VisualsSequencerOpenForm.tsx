import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { SequencerService } from "@/applications/visuals-sequencer/services/sequencer";
import { AssetRootFormRow } from "@/core/assets/components/AssetRootFormRow";
import { useAssetRootField } from "@/core/assets/lib";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";

interface IVisualsSequencerOpenFormProps extends BaseComponentProps {
  /** Called once an open attempt has finished, successfully or not. */
  onFinished?: () => void;
}

/**
 * The way into the sequencer: one model, whose motions a track is written out of.
 */
export function VisualsSequencerOpenForm({ onFinished }: IVisualsSequencerOpenFormProps): ReactElement {
  const sequencerService: SequencerService = useInjection(SequencerService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = sequencerService.visual.isLoading;

  const visual: IPathField = usePathField({
    application: EApplicationId.VISUALS_SEQUENCER,
    id: "visual",
    title: "Select ogf visual",
    filters: [{ name: "Ogf visual", extensions: ["ogf"] }],
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
