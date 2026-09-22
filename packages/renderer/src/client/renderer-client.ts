import { IRendererDevice } from "#/contract/renderer-device";
import {
  ERendererRequest,
  ERendererResponse,
  IRendererConfiguration,
  listRendererTransfers,
  TRendererRequest,
  TRendererResponse,
} from "#/contract/renderer-messages";
import { IRendererReport } from "#/contract/renderer-report";
import { IOffscreenRenderSize } from "#/frame/offscreen-render-target";
import { IRenderTarget } from "#/frame/render-target";

/**
 * What a consumer hands the renderer, and what it is told back.
 */
export interface IRendererClientOptions {
  /** The renderer's worker, from `createRendererWorker`. */
  worker: Worker;
  /** The page canvas whose drawing is handed over. */
  target: IRenderTarget;
  configuration: IRendererConfiguration;
  onReady?: (device: IRendererDevice) => void;
  onFailed?: (reason: string) => void;
  onReport?: (report: IRendererReport) => void;
}

/**
 * The renderer, from the page that owns its canvas.
 */
export class RendererClient {
  private readonly worker: Worker;
  private readonly target: IRenderTarget;
  private readonly unobserve: () => void;

  public constructor({ worker, target, configuration, onReady, onFailed, onReport }: IRendererClientOptions) {
    if (!(target.canvas instanceof HTMLCanvasElement)) {
      throw new Error("The renderer draws on a page canvas, and this target holds none.");
    }

    this.worker = worker;
    this.target = target;

    this.worker.onmessage = (event: MessageEvent<TRendererResponse>): void => {
      const response: TRendererResponse = event.data;

      switch (response.kind) {
        case ERendererResponse.READY:
          return onReady?.(response.device);

        case ERendererResponse.FAILED:
          return onFailed?.(response.reason);

        case ERendererResponse.REPORT:
          return onReport?.(response.report);
      }
    };

    this.worker.onerror = (event: ErrorEvent): void => onFailed?.(`The renderer worker failed: ${event.message}`);

    this.post({
      canvas: target.canvas.transferControlToOffscreen(),
      configuration,
      kind: ERendererRequest.START,
      ...this.getSize(),
    });

    this.unobserve = target.observe(() => this.post({ kind: ERendererRequest.RESIZE, ...this.getSize() }));
  }

  /**
   * @param configuration - How frames are drawn from now on.
   */
  public configure(configuration: IRendererConfiguration): void {
    this.post({ configuration, kind: ERendererRequest.CONFIGURE });
  }

  /** Stops the renderer and its thread. */
  public dispose(): void {
    this.unobserve();
    this.post({ kind: ERendererRequest.DISPOSE });
    this.worker.terminate();
  }

  private post(request: TRendererRequest): void {
    this.worker.postMessage(request, listRendererTransfers(request));
  }

  private getSize(): IOffscreenRenderSize {
    return { height: this.target.height, pixelRatio: this.target.pixelRatio, width: this.target.width };
  }
}
