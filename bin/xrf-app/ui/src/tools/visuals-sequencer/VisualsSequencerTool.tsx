import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback, useState } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { VisualPreviewLayout } from "@/core/visuals/components/preview/VisualPreviewLayout";
import { IOpenVisual } from "@/core/visuals/services";
import { VisualLoadService } from "@/core/visuals/services/visual-load";
import { VisualsSequencerService } from "@/tools/visuals-sequencer/services/visuals-sequencer";

import { SEQUENCER_PANELS } from "./components/panels";
import { SequencerTransport } from "./components/SequencerTransport";
import { VisualsSequencerOpenForm } from "./components/VisualsSequencerOpenForm";

/**
 * Compose an ordered track out of one visual's motions and watch it play.
 */
export function VisualsSequencerTool({
  "data-testid": dataTestId = "visuals-sequencer-tool",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const service: VisualsSequencerService = useInjection(VisualsSequencerService);
  const loadService: VisualLoadService = useInjection(VisualLoadService);

  const [isPickerOpen, setPickerOpen] = useState(false);

  const visual: Nullable<IOpenVisual> = service.visual.value;

  const onBack = useCallback(() => setPickerOpen(true), []);

  const onFinished = useCallback(() => setPickerOpen(false), []);

  return (
    <SessionTool
      root={loadService.selection}
      isOpen={!isPickerOpen && visual !== null}
      openForm={<VisualsSequencerOpenForm onFinished={onFinished} />}
      workspace={
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
      }
    />
  );
}
