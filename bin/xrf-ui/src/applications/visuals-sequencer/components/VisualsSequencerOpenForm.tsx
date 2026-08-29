import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { SequencerService } from "@/applications/visuals-sequencer/services/sequencer";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
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
  const pathsService: PathsService = useInjection(PathsService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const isLoading: boolean = sequencerService.visual.isLoading;

  const seed = useCallback(() => resolveExistingPathRole(EPathRole.VISUALS, pathsService.paths), [pathsService.paths]);

  const visual: IPathField = usePathField({
    application: EApplicationId.VISUALS_SEQUENCER,
    id: "visual",
    title: "Select ogf visual",
    filters: [{ name: "Ogf visual", extensions: ["ogf"] }],
    isDisabled: isLoading,
    seed,
  });

  const onOpen = useCallback(async () => {
    if (!visual.value) {
      log.info("Cannot open a visual without a path");

      return;
    }

    await sequencerService.openFile(visual.value);

    onFinished?.();
  }, [log, onFinished, sequencerService, visual.value]);

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
    </PickerForm>
  );
}
