import { IRenderSurfaceHost } from "@/core/render/lib/surface/render-surface-host";
import { Nullable } from "@/lib/types/general";

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

  /** Whether there is currently somewhere to draw. */
  protected get isAttached(): boolean {
    return this.container !== null;
  }

  /**
   * Builds it again against the element it already has.
   */
  protected remount(): void {
    const container: Nullable<HTMLElement> = this.container;

    if (container) {
      this.unmount();
      this.mount(container);
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
