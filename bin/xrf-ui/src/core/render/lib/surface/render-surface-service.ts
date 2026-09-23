import { Nullable } from "@xrf/types";

import { IRenderSurfaceHost } from "@/core/render/lib/surface/render-surface-host";

/**
 * A host that builds whatever draws, when it is given somewhere to draw.
 */
export abstract class RenderSurfaceService implements IRenderSurfaceHost {
  private container: Nullable<HTMLElement> = null;

  /**
   * Takes somewhere to draw, and builds.
   *
   * @param container - The element the viewport fills.
   */
  public attach(container: HTMLElement): void {
    this.detach();

    this.container = container;
    this.mount(container);
  }

  /** Releases whatever was built, and forgets where it was. */
  public detach(): void {
    if (this.container) {
      this.unmount();
      this.container = null;
    }
  }

  /**
   * Builds whatever draws, and starts telling it things.
   *
   * @param container - The element the viewport fills.
   */
  protected abstract mount(container: HTMLElement): void;

  /** Releases what `mount` built, and stops telling it anything. */
  protected abstract unmount(): void;
}
