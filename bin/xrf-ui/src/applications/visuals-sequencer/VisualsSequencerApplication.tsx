import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { SequencerService } from "@/applications/visuals-sequencer/services/sequencer";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { VisualPreviewLayout } from "@/core/visuals/components/preview/VisualPreviewLayout";
import { IOpenVisual } from "@/core/visuals/services";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { SEQUENCER_PANELS } from "./components/panels";
import { SequencerTransport } from "./components/SequencerTransport";
import { VisualsSequencerOpenForm } from "./components/VisualsSequencerOpenForm";

/**
 * Compose an ordered track out of one visual's motions and watch it play.
 */
export function VisualsSequencerApplication({
  "data-testid": dataTestId = "visuals-sequencer-application",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const service: SequencerService = useInjection(SequencerService);

  const [isPickerOpen, setPickerOpen] = useState(false);

  const visual: Nullable<IOpenVisual> = service.visual.value;

  const onBack = useCallback(() => setPickerOpen(true), []);

  const onFinished = useCallback(() => setPickerOpen(false), []);

  if (!service.isReady) {
    return <ApplicationLoader />;
  }

  if (isPickerOpen || !visual) {
    return <VisualsSequencerOpenForm onFinished={onFinished} />;
  }

  return (
    <VisualPreviewLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      name={service.sourceLabel}
      panels={SEQUENCER_PANELS}
      footer={<SequencerTransport />}
      isLoading={service.visual.isLoading}
      error={service.visual.error?.message}
      onBack={onBack}
    />
  );
}
