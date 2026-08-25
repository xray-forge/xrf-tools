import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { IVisualPreviewViewportProps, VisualPreviewViewport } from "@/core/visuals/components/preview";

/**
 * The viewport posed by the track rather than by a single picked motion.
 */
export function SequencerViewport(props: IVisualPreviewViewportProps): ReactElement {
  const service: VisualSequenceService = useInjection(VisualSequenceService);

  return (
    <VisualPreviewViewport
      {...props}
      motionTransforms={service.transforms}
      motionFrame={service.frame}
      motionFloatsPerBone={service.floatsPerBone}
    />
  );
}
