import { OnDeactivation } from "@wirestate/core";
import { comparer, reaction, RefObservable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { renderCommands } from "@/core/ipc/commands/render";
import {
  ERenderPresentation,
  RenderCameraPose,
  RenderFrameReport,
  RenderLoadReport,
  RenderSettings,
} from "@/core/ipc/types/xrf-renderer";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { NativeViewportTarget } from "@/core/render/lib/native/native-viewport-target";
import { IRenderSharedSettings } from "@/core/render/lib/settings/render-shared-settings";
import { IRenderSurfaceHost } from "@/core/render/lib/surface/render-surface-host";
import { SettingsService } from "@/core/settings/services/settings";
import { Logger } from "@/lib/logging";

/**
 * @param shared - What the application sets for every viewport.
 * @returns What the native renderer draws every viewport with.
 */
export function toNativeRenderSettings(shared: IRenderSharedSettings): RenderSettings {
  // todo: Pace a native viewport to a rate below the display's, which only an unlimited rate skips today.
  return {
    presentation: shared.pacing.rateLimit === "unlimited" ? ERenderPresentation.UNCAPPED : ERenderPresentation.VSYNC,
  };
}

/**
 * A service owning one native viewport: attached where a view hands it an element, told everything through reactions
 * while attached, and let go when the view or the application goes.
 */
export abstract class NativeRenderSurfaceService implements IRenderSurfaceHost {
  public abstract readonly log: Logger;

  /** Why the viewport cannot be drawn, or null while it draws: kept until a view is attached again. */
  @RefObservable()
  public failure: Nullable<string> = null;

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
      onCamera: (pose: RenderCameraPose): void => this.onCamera(pose),
      onFailed: (message: string): void => this.fail(message),
      onFrame: (report: RenderFrameReport): void => this.onFrame(report),
      onLoad: (report: RenderLoadReport): void => this.onLoad(report),
    });

    this.viewport = viewport;
    this.target = new NativeViewportTarget(container, viewport);
    this.onAttached(container);
    this.reactions.push(
      reaction(
        () => toNativeRenderSettings(this.settingsService.sharedRenderSettings),
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
   * @param report - What the viewport's recent frames cost.
   */
  protected abstract onFrame(report: RenderFrameReport): void;

  /**
   * @param _pose - Where the viewport's camera stands now.
   */
  protected onCamera(_pose: RenderCameraPose): void {}

  /**
   * @param _report - How far the viewport's scene has loaded.
   */
  protected onLoad(_report: RenderLoadReport): void {}

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
