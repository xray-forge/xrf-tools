import { OnDeactivation } from "@wirestate/core";
import { comparer, reaction, RefObservable, runInAction } from "@wirestate/mobx";
import {
  ERenderResolution,
  IDdsRefusal,
  IRendererLighting,
  IRendererReport,
  IRendererSettings,
  IRendererTextureFetch,
  RendererClient,
  TRendererOverlay,
} from "@xrf/renderer";
import { createRendererWorker } from "@xrf/renderer/worker";
import { Nullable } from "@xrf/types";

import { IPC_METRICS } from "@/core/ipc/metrics";
import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { IRenderSurfaceHost } from "@/core/render/lib/surface/render-surface-host";
import { SettingsService } from "@/core/settings/services/settings";
import { Logger } from "@/lib/logging";

/** What the renderer's own fetches of textures are counted under, beside the page's bulk fetches. */
const FETCHED_TEXTURE_METRIC: string = "renderer|fetch_texture";

/**
 * A service owning one renderer: started on first use, drawn wherever a view attaches it, told everything through
 * reactions, started again for the next view once it fails, and let go when the application deactivates.
 */
export abstract class RenderSurfaceService implements IRenderSurfaceHost {
  public abstract readonly log: Logger;

  /** Why the renderer stopped, or null while it draws: kept until a view is attached again. */
  @RefObservable()
  public failure: Nullable<string> = null;

  protected client: Nullable<RendererClient> = null;
  protected target: Nullable<DomRenderTarget> = null;

  private readonly reactions: Array<() => void> = [];
  /** What each frame helper was last put as, so a toggle that leaves one alone does not send it again. */
  private readonly framed: Map<string, Nullable<string>> = new Map();
  /** What the renderer was last told its settings and its lighting are, so a change neither reads sends neither. */
  private sentSettings: Nullable<IRendererSettings> = null;
  private sentLighting: Nullable<IRendererLighting> = null;

  protected constructor(protected readonly settingsService: SettingsService) {}

  /**
   * Takes somewhere to draw, and draws there.
   *
   * @param container - The element the viewport fills.
   */
  public attach(container: HTMLElement): void {
    this.detach();

    // A renderer that failed draws nothing more: a view shown again is drawn by another.
    if (this.failure !== null) {
      this.stop();
    }

    const target: DomRenderTarget = new DomRenderTarget(container, this.settingsService.renderResolution);

    this.target = target;
    this.ensureClient().attach(target);
    this.onAttached();
  }

  /** Stops drawing, and releases the canvas; the renderer and what it holds stay. */
  public detach(): void {
    const target: Nullable<DomRenderTarget> = this.target;

    if (!target) {
      return;
    }

    this.target = null;
    this.client?.detach();
    // Released here because it was made here: the canvas on the page is this service's.
    target.dispose();
    this.onDetached();
  }

  /** Releases the renderer and stops telling it anything. */
  @OnDeactivation()
  public dispose(): void {
    this.detach();
    this.stop();
  }

  /** @returns The settings the renderer draws with now. */
  protected abstract toSettings(): IRendererSettings;

  /**
   * Tells a new renderer what is open now, and again whenever any of it changes.
   *
   * @param client - The renderer, just started.
   * @returns What stops each reaction.
   */
  protected abstract start(client: RendererClient): Array<() => void>;

  /**
   * @param report - What the renderer reported of its last frames.
   */
  protected abstract onReport(report: IRendererReport): void;

  /**
   * @param key - The texture refused.
   * @param refusal - Why its file was refused as stored.
   */
  protected onTextureRefused(key: string, refusal: IDdsRefusal): void {
    this.log.warn(`Texture '${key}' was refused by the renderer:`, refusal.detail);
  }

  /**
   * Counts what a texture the renderer fetched cost, the way the page's own fetches are counted.
   *
   * @param key - The texture fetched.
   * @param fetch - What it came to.
   */
  protected onTextureFetched(key: string, fetch: IRendererTextureFetch): void {
    IPC_METRICS.record(FETCHED_TEXTURE_METRIC, fetch.duration, fetch.bytes, fetch.failure !== null);

    if (fetch.failure) {
      this.log.warn(`Texture '${key}' could not be fetched by the renderer:`, fetch.failure);
    }
  }

  /** Called once a view is drawn into. */
  protected onAttached(): void {}

  /** Called once a view is no longer drawn into. */
  protected onDetached(): void {}

  /** Forgets what the subclass told the renderer, which went with it. */
  protected release(): void {}

  /** Sends the settings, unless they read as what was last sent. */
  protected sendSettings(): void {
    const next: IRendererSettings = this.toSettings();

    if (this.client && !(this.sentSettings && comparer.structural(next, this.sentSettings))) {
      this.sentSettings = next;
      this.client.configure(next);
    }
  }

  /**
   * Sends the lighting, unless it reads as what was last sent.
   *
   * @param lighting - How the scene is lit from now on.
   */
  protected sendLighting(lighting: IRendererLighting): void {
    if (this.client && !(this.sentLighting && comparer.structural(lighting, this.sentLighting))) {
      this.sentLighting = lighting;
      this.client.setLighting(lighting);
    }
  }

  /**
   * Puts a frame helper when what it is built from changed, and releases it when it is no longer shown.
   *
   * @param key - What the helper is held under.
   * @param state - What it is built from, or null to release it.
   * @param build - Builds it, called only when it is put.
   */
  protected putFrame(key: string, state: Nullable<string>, build: () => TRendererOverlay): void {
    if (!this.client || (this.framed.get(key) ?? null) === state) {
      return;
    }

    this.framed.set(key, state);

    if (state === null) {
      this.client.releaseOverlay(key);
    } else {
      this.client.putOverlay(key, build());
    }
  }

  /** The renderer, started on first use and told everything from then on. */
  protected ensureClient(): RendererClient {
    if (this.client) {
      return this.client;
    }

    const settings: IRendererSettings = this.toSettings();
    const client: RendererClient = new RendererClient({
      onFailed: (reason: string): void => this.fail(reason),
      onReport: (report: IRendererReport): void => this.onReport(report),
      onTextureFetched: (key: string, fetch: IRendererTextureFetch): void => this.onTextureFetched(key, fetch),
      onTextureRefused: (key: string, refusal: IDdsRefusal): void => this.onTextureRefused(key, refusal),
      settings,
      worker: createRendererWorker(),
    });

    this.client = client;
    this.sentSettings = settings;

    this.reactions.push(
      reaction(
        () => [this.settingsService.rendererChoice, this.settingsService.framePacing],
        () => this.sendSettings()
      ),
      reaction(
        () => this.settingsService.renderResolution,
        (resolution: ERenderResolution) => this.target?.setResolution(resolution)
      ),
      ...this.start(client)
    );

    return client;
  }

  /** Lets the renderer go with everything told to it, so the next one starts from nothing. */
  private stop(): void {
    this.reactions.splice(0).forEach((stop: () => void) => stop());
    this.client?.dispose();
    this.client = null;
    this.framed.clear();
    this.sentSettings = null;
    this.sentLighting = null;
    this.release();

    runInAction(() => {
      this.failure = null;
    });
  }

  private fail(reason: string): void {
    this.log.error("The renderer failed:", reason);

    runInAction(() => {
      this.failure = reason;
    });
  }
}
