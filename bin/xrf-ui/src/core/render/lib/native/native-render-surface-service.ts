import { OnDeactivation } from "@wirestate/core";
import { comparer, reaction, RefObservable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { renderCommands } from "@/core/ipc/commands/render";
import {
  RenderAppliedReport,
  RenderCameraPose,
  RenderFrameReport,
  RenderLoadReport,
  RenderSettings,
  RenderWeatherReport,
} from "@/core/ipc/types/xrf-renderer";
import { EMPTY_RENDER_FRAME_REPORT } from "@/core/render/lib/native/native-frame-report";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { NativeViewportTarget } from "@/core/render/lib/native/native-viewport-target";
import { IRenderSurfaceHost } from "@/core/render/lib/surface/render-surface-host";
import { SettingsService } from "@/core/settings/services/settings";
import { Logger } from "@/lib/logging";

/**
 * A service owning one native viewport: attached where a view hands it an element, told everything through reactions
 * while attached, and let go when the view or the application goes.
 */
export abstract class NativeRenderSurfaceService implements IRenderSurfaceHost {
  public abstract readonly log: Logger;

  /** Why the viewport cannot be drawn, or null while it draws: kept until a view is attached again. */
  @RefObservable()
  public failure: Nullable<string> = null;

  /** What the viewport's recent frames cost, for the readouts; empty while none is attached. */
  @RefObservable()
  public frame: RenderFrameReport = EMPTY_RENDER_FRAME_REPORT;

  /** What the renderer draws the viewport's frames with, as it resolved what it was asked; null until it says. */
  @RefObservable()
  public applied: Nullable<RenderAppliedReport> = null;

  protected viewport: Nullable<NativeViewport> = null;

  private target: Nullable<NativeViewportTarget> = null;
  private readonly reactions: Array<() => void> = [];

  protected constructor(protected readonly settingsService: SettingsService) {}

  /**
   * Draws a native viewport under an element.
   *
   * @param container - The element the viewport fills.
   */
  public attach(container: HTMLElement): void {
    this.detach();

    runInAction(() => {
      this.failure = null;
    });

    const viewport: NativeViewport = new NativeViewport({
      onApplied: (report: RenderAppliedReport): void => {
        runInAction(() => {
          this.applied = report;
        });
      },
      onCamera: (pose: RenderCameraPose): void => this.onCamera(pose),
      onFailed: (message: string): void => this.fail(message),
      onFrame: (report: RenderFrameReport): void => {
        runInAction(() => {
          this.frame = report;
        });
        this.onFrame(report);
      },
      onLoad: (report: RenderLoadReport): void => this.onLoad(report),
      onWeather: (report: Nullable<RenderWeatherReport>): void => this.onWeather(report),
    });

    this.viewport = viewport;
    this.target = new NativeViewportTarget(container, viewport);
    this.onAttached(container);
    this.reactions.push(
      reaction(
        () => this.settingsService.renderSettings,
        (settings: RenderSettings) => void renderCommands.configure(settings).catch(() => {}),
        { equals: comparer.structural, fireImmediately: true }
      ),
      ...this.start(viewport)
    );
  }

  /** Stops drawing and closes the element's hole. */
  public detach(): void {
    if (this.viewport) {
      this.onDetached();
    }

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
  protected onCamera(_pose: RenderCameraPose): void {}

  /**
   * @param _report - How far the viewport's scene has loaded.
   */
  protected onLoad(_report: RenderLoadReport): void {}

  /**
   * @param _report - Where the viewport's weather stands, or null while none plays.
   */
  protected onWeather(_report: Nullable<RenderWeatherReport>): void {}

  /**
   * Called once a viewport draws under an element.
   *
   * @param _container - The element it draws under.
   */
  protected onAttached(_container: HTMLElement): void {}

  /** Called before the viewport goes. */
  protected onDetached(): void {}

  /** Forgets what the subclass told the viewport, which went with it. */
  protected release(): void {}

  private fail(message: string): void {
    this.log.error("The native viewport cannot be drawn:", message);

    runInAction(() => {
      this.failure = message;
    });
  }
}
