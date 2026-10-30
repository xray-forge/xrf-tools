import { OnDeactivation } from "@wirestate/core";
import { Computed, RefObservable, runInAction } from "@wirestate/mobx";
import { transformError } from "@xrf/errors";
import { RenderAppliedReport, RenderFrameReport, RenderLoadReport } from "@xrf/ipc/types/xrf-renderer";
import { WorldCameraPose, WorldWeatherReport } from "@xrf/ipc/types/xrf-world";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { EMPTY_RENDER_FRAME_REPORT } from "@/core/render/lib/native/native-frame-report";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { NativeViewportTarget } from "@/core/render/lib/native/native-viewport-target";
import { IRenderSurfaceHost } from "@/core/render/lib/surface/render-surface-host";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

/**
 * A service owning one native viewport: attached where a view hands it an element, told everything through reactions
 * while attached, and let go when the view or the tool goes.
 */
export abstract class NativeRenderSurfaceService implements IRenderSurfaceHost {
  public abstract readonly log: Logger;

  /** Why the viewport cannot be drawn, or null while it draws: kept until a view is attached again. */
  @RefObservable()
  private drawFailure: Nullable<string> = null;

  /** Why what was last asked to be shown is not, or null: kept until another show is asked. */
  @RefObservable()
  private showFailure: Nullable<string> = null;

  /** What the viewport's recent frames cost, for the readouts; empty while none is attached. */
  @RefObservable()
  public frame: RenderFrameReport = EMPTY_RENDER_FRAME_REPORT;

  /** What the renderer draws the viewport's frames with, as it resolved what it was asked; null until it says. */
  @RefObservable()
  public applied: Nullable<RenderAppliedReport> = null;

  protected viewport: Nullable<NativeViewport> = null;

  private target: Nullable<NativeViewportTarget> = null;
  private readonly reactions: Array<() => void> = [];
  /** Shows asked, so one replaced since neither hands its scene on nor fails. */
  private showing: number = 0;

  protected constructor(protected readonly renderSettingsService: RenderSettingsService) {}

  /** Why the viewport draws nothing it was asked to, or null while it does: it cannot be drawn, or the show failed. */
  @Computed()
  public get failure(): Nullable<string> {
    return this.drawFailure ?? this.showFailure;
  }

  /**
   * Draws a native viewport under an element.
   *
   * @param container - The element the viewport fills.
   */
  public attach(container: HTMLElement): void {
    this.detach();

    runInAction(() => {
      this.drawFailure = null;
      this.showFailure = null;
    });

    const viewport: NativeViewport = new NativeViewport({
      onApplied: (report: RenderAppliedReport): void => {
        runInAction(() => {
          this.applied = report;
        });
      },
      onCamera: (pose: WorldCameraPose): void => this.onCamera(pose),
      onCommandFailed: (command: string, error: unknown): void => {
        this.log.warn("A native viewport command failed:", command, error);
      },
      onFailed: (message: string): void => this.fail(message),
      onFrame: (report: RenderFrameReport): void => {
        // A viewport reports only frames it drew, so by its first the window holds the page's backdrop.
        this.target?.openHole();
        runInAction(() => {
          this.frame = report;
        });
        this.onFrame(report);
      },
      onLoad: (report: RenderLoadReport): void => this.onLoad(report),
      onWeather: (report: Nullable<WorldWeatherReport>): void => this.onWeather(report),
    });

    this.viewport = viewport;
    this.target = new NativeViewportTarget(
      container,
      viewport,
      (point: IRenderViewPoint, isThrough: boolean, isToggling: boolean) => this.onClick(point, isThrough, isToggling)
    );
    this.onAttached(container);
    this.reactions.push(...this.start(viewport));
  }

  /** Stops drawing and closes the element's hole. */
  public detach(): void {
    if (this.viewport) {
      this.onDetached();
    }

    this.showing += 1;
    this.reactions.splice(0).forEach((stop: () => void) => stop());
    this.target?.dispose();
    this.target = null;
    this.viewport?.dispose();
    this.viewport = null;
    this.release();

    runInAction(() => {
      this.frame = EMPTY_RENDER_FRAME_REPORT;
      this.applied = null;
    });
  }

  @OnDeactivation()
  public dispose(): void {
    this.detach();
  }

  /**
   * Tells a viewport just attached what is open now, and again whenever any of it changes.
   *
   * @param viewport - The viewport, just attached.
   * @returns What stops each reaction.
   */
  protected abstract start(viewport: NativeViewport): Array<() => void>;

  /**
   * @param _report - What the viewport's recent frames cost, as `frame` now holds it.
   */
  protected onFrame(_report: RenderFrameReport): void {}

  /**
   * @param _pose - Where the viewport's camera stands now.
   */
  protected onCamera(_pose: WorldCameraPose): void {}

  /**
   * @param _report - How far the viewport's scene has loaded.
   */
  protected onLoad(_report: RenderLoadReport): void {}

  /**
   * @param _report - Where the viewport's weather stands, or null while none plays.
   */
  protected onWeather(_report: Nullable<WorldWeatherReport>): void {}

  /**
   * Called once a viewport draws under an element.
   *
   * @param _container - The element it draws under.
   */
  protected onAttached(_container: HTMLElement): void {}

  /**
   * Called for a click on the viewport, the second of a double click aside.
   *
   * @param _point - Where, in css pixels from the viewport's top left corner.
   * @param _isThrough - Whether Alt was held, which picks through glass.
   * @param _isToggling - Whether Ctrl (Cmd on macOS) was held, which adds to the selection or takes from it.
   */
  protected onClick(_point: IRenderViewPoint, _isThrough: boolean, _isToggling: boolean): void {}

  /** Called before the viewport goes. */
  protected onDetached(): void {}

  /**
   * Asks the viewport to show something, handing on the scene it draws; a show that fails is held as the failure, and
   * one replaced since does neither.
   *
   * @param send - Sends the show to the viewport.
   * @param onShown - Takes the scene drawn, or null for nothing shown or no viewport attached.
   */
  protected show(
    send: (viewport: NativeViewport) => Promise<Nullable<number>>,
    onShown: (scene: Nullable<number>) => void
  ): void {
    const showing: number = ++this.showing;

    runInAction(() => {
      this.showFailure = null;
    });

    if (!this.viewport) {
      return;
    }

    send(this.viewport).then(
      (scene: Nullable<number>) => {
        if (showing === this.showing) {
          onShown(scene);
        }
      },
      (error: unknown) => {
        if (showing === this.showing) {
          const message: string = transformError(error).message;

          this.log.error("The native viewport cannot show what it was asked:", message);

          runInAction(() => {
            this.showFailure = message;
          });
        }
      }
    );
  }

  /** Forgets what the subclass told the viewport, which went with it. */
  protected release(): void {}

  private fail(message: string): void {
    this.log.error("The native viewport cannot be drawn:", message);

    runInAction(() => {
      this.drawFailure = message;
    });
  }
}
