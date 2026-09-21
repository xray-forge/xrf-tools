import { ReactElement, useEffect, useRef } from "react";

import { IRenderSurfaceHost } from "@/core/render/lib/surface/render-surface-host";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IRenderSurfaceProps extends BaseComponentProps {
  /** Whatever draws here, which is told where and nothing else. */
  host: IRenderSurfaceHost;
}

/**
 * Somewhere to draw.
 */
export function RenderSurface({
  "data-testid": dataTestId = "render-surface",
  id,
  className,
  host,
}: IRenderSurfaceProps): ReactElement {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (containerRef.current) {
      host.attach(containerRef.current);
    }

    return () => host.detach();
  }, [host]);

  return <div data-testid={dataTestId} id={id} className={cn(className, "h-full w-full")} ref={containerRef} />;
}
