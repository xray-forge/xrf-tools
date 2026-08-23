import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import {
  IVisualPreviewViewportProps,
  VisualPreviewViewport,
} from "@/core/visuals/components/preview/VisualPreviewViewport";
import { IPosedMotion, VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { Nullable } from "@/lib/types/general";

/**
 * The viewport with the playing motion applied to it.
 *
 * A frame lasts a thirtieth of a second, and this is the only component that reads which one is current: reading it
 * higher up would re-render the toolbar, the panel stripe and the playback bar thirty times a second to move a
 * skeleton. Everything else about the viewport still arrives as props.
 */
export function VisualPreviewMotionViewport(props: IVisualPreviewViewportProps): ReactElement {
  const service: VisualMotionService = useInjection(VisualMotionService);

  const posed: Nullable<IPosedMotion> = service.posed.value;

  return (
    <VisualPreviewViewport
      {...props}
      motionTransforms={posed?.transforms ?? null}
      motionFrame={service.frame}
      motionFloatsPerBone={service.floatsPerBone}
    />
  );
}
