import { useInjection } from "@wirestate/react";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { LevelRenderService } from "@/core/level/services/level-render";
import { RenderSurface } from "@/core/render/components/RenderSurface";

export type ILevelPreviewViewportProps = BaseComponentProps;

/** Somewhere to draw the open level, which is the render service bound to a surface. */
export function LevelPreviewViewport({
  "data-testid": dataTestId = "level-preview-viewport",
  id,
  className,
}: ILevelPreviewViewportProps): ReactElement {
  return (
    <RenderSurface data-testid={dataTestId} id={id} className={className} host={useInjection(LevelRenderService)} />
  );
}
