import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useRef } from "react";

import { LevelRenderService } from "@/core/level/services";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

export type ILevelPreviewViewportProps = BaseComponentProps;

/**
 * Says where the level is drawn, and nothing else.
 */
export function LevelPreviewViewport({
  "data-testid": dataTestId = "level-preview-viewport",
  id,
  className,
}: ILevelPreviewViewportProps): ReactElement {
  const containerRef = useRef<HTMLDivElement>(null);
  const renderService: LevelRenderService = useInjection(LevelRenderService);

  useEffect(() => {
    if (containerRef.current) {
      renderService.attach(containerRef.current);
    }

    return () => renderService.detach();
  }, [renderService]);

  return <div ref={containerRef} data-testid={dataTestId} id={id} className={cn(className, "h-full w-full")} />;
}
