import { Nullable } from "@xrf/types";

import { TRendererCamera, TRendererCameraCommand } from "#/contract/renderer-camera";
import { TRendererCaptureSource } from "#/contract/renderer-capture";
import { IRendererDevice } from "#/contract/renderer-device";
import { IRendererLighting } from "#/contract/renderer-lighting";
import {
  ERendererRequest,
  ERendererResponse,
  listRendererTransfers,
  TRendererRequest,
  TRendererResponse,
} from "#/contract/renderer-messages";
import { IRendererReport } from "#/contract/renderer-report";
import { IRendererSettings } from "#/contract/renderer-settings";
import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { IRendererObject } from "#/contract/scene/renderer-object";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { IOffscreenRenderSize } from "#/frame/offscreen-render-target";
import { IRenderTarget } from "#/frame/render-target";
import { RenderInputForwarder } from "#/input/render-input-forwarder";
import { IDdsRefusal } from "#/texture/dds/dds-refusal";

/**
 * What a consumer hands the renderer, and what it is told back.
 */
export interface IRendererClientOptions {
  /** The renderer's worker, from `createRendererWorker`. */
  worker: Worker;
  settings: IRendererSettings;
  onReady?: (device: IRendererDevice) => void;
  onFailed?: (reason: string) => void;
  onReport?: (report: IRendererReport) => void;
  /** A texture's file was refused as stored; putting its decoded picture under the same key fills the slot. */
  onTextureRefused?: (key: string, refusal: IDdsRefusal) => void;
}

/** A canvas showing the renderer's frames, and what watches it on the page. */
interface IRendererClientView {
  target: IRenderTarget;
  input: RenderInputForwarder;
  unobserve: () => void;
}

/**
 * The renderer, from the page: one device for as long as the client lives, and a canvas while one is attached.
 */
export class RendererClient {
  private static getSize(target: IRenderTarget): IOffscreenRenderSize {
    return { height: target.height, pixelRatio: target.pixelRatio, width: target.width };
  }

  private readonly worker: Worker;
  private readonly captures: Map<number, (image: Nullable<ImageBitmap>) => void> = new Map();
  private view: Nullable<IRendererClientView> = null;
  private captureId: number = 0;

  public constructor({ worker, settings, onReady, onFailed, onReport, onTextureRefused }: IRendererClientOptions) {
    this.worker = worker;

    this.worker.onmessage = (event: MessageEvent<TRendererResponse>): void => {
      const response: TRendererResponse = event.data;

      switch (response.kind) {
        case ERendererResponse.READY:
          return onReady?.(response.device);

        case ERendererResponse.FAILED:
          return onFailed?.(response.reason);

        case ERendererResponse.REPORT:
          return onReport?.(response.report);

        case ERendererResponse.TEXTURE_REFUSED:
          return onTextureRefused?.(response.key, response.refusal);

        case ERendererResponse.CURSOR:
          return this.view?.input.setCursor(response.cursor);

        case ERendererResponse.CAPTURED: {
          const resolve = this.captures.get(response.id);

          this.captures.delete(response.id);

          return resolve ? resolve(response.image) : response.image?.close();
        }
      }
    };

    this.worker.onerror = (event: ErrorEvent): void => onFailed?.(`The renderer worker failed: ${event.message}`);

    this.post({ kind: ERendererRequest.START, settings });
  }

  /**
   * Shows frames on a page canvas, handing its drawing over for good: a canvas is transferred once.
   *
   * @param target - The canvas and its size.
   */
  public attach(target: IRenderTarget): void {
    if (!(target.canvas instanceof HTMLCanvasElement)) {
      throw new Error("The renderer draws on a page canvas, and this target holds none.");
    }

    this.detach();

    const canvas: HTMLCanvasElement = target.canvas;

    this.view = {
      input: new RenderInputForwarder(canvas, (event) => this.post({ event, kind: ERendererRequest.INPUT })),
      target,
      unobserve: target.observe(() => this.post({ kind: ERendererRequest.RESIZE, ...RendererClient.getSize(target) })),
    };

    this.post({
      canvas: canvas.transferControlToOffscreen(),
      kind: ERendererRequest.ATTACH_VIEW,
      ...RendererClient.getSize(target),
    });
  }

  /** Stops showing frames; textures, geometry and captures carry on. */
  public detach(): void {
    const view: Nullable<IRendererClientView> = this.view;

    if (!view) {
      return;
    }

    this.view = null;
    view.unobserve();
    view.input.dispose();
    this.post({ kind: ERendererRequest.DETACH_VIEW });
  }

  /**
   * @param settings - How frames are drawn from now on.
   */
  public configure(settings: IRendererSettings): void {
    this.post({ kind: ERendererRequest.CONFIGURE, settings });
  }

  /**
   * @param key - What the texture is held under; a surface names it by this.
   * @param source - Its bytes, moved to the renderer and no longer usable here.
   */
  public putTexture(key: string, source: TRendererTextureSource): void {
    this.post({ key, kind: ERendererRequest.PUT_TEXTURE, source });
  }

  public releaseTexture(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_TEXTURE });
  }

  /**
   * @param key - What the geometry is held under; an object names it by this.
   * @param geometry - Its arrays, moved to the renderer and no longer usable here.
   */
  public putGeometry(key: string, geometry: IRendererGeometry): void {
    this.post({ geometry, key, kind: ERendererRequest.PUT_GEOMETRY });
  }

  public releaseGeometry(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_GEOMETRY });
  }

  public putSurface(key: string, surface: IRendererSurface): void {
    this.post({ key, kind: ERendererRequest.PUT_SURFACE, surface });
  }

  public releaseSurface(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_SURFACE });
  }

  public putObject(key: string, object: IRendererObject): void {
    this.post({ key, kind: ERendererRequest.PUT_OBJECT, object });
  }

  public releaseObject(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_OBJECT });
  }

  /**
   * @param lighting - How the scene is lit from now on.
   */
  public setLighting(lighting: IRendererLighting): void {
    this.post({ kind: ERendererRequest.LIGHTING, lighting });
  }

  /**
   * @param camera - The camera wanted, from where it starts.
   */
  public setCamera(camera: TRendererCamera): void {
    this.post({ camera, kind: ERendererRequest.CAMERA });
  }

  /**
   * @param command - What to do with the camera.
   */
  public commandCamera(command: TRendererCameraCommand): void {
    this.post({ command, kind: ERendererRequest.CAMERA_COMMAND });
  }

  /**
   * Draws a picture of the frame or of a texture.
   *
   * @param source - What to draw: a frame view, at the canvas's drawing size, or a bump plane at its own.
   * @returns The picture, or null where there was nothing to draw, such as a frame with no view attached.
   */
  public capture(source: TRendererCaptureSource): Promise<Nullable<ImageBitmap>> {
    const id: number = ++this.captureId;

    return new Promise((resolve) => {
      this.captures.set(id, resolve);
      this.post({ id, kind: ERendererRequest.CAPTURE, source });
    });
  }

  /** Stops the renderer and its thread. */
  public dispose(): void {
    this.detach();
    this.post({ kind: ERendererRequest.DISPOSE });
    this.worker.terminate();
    this.captures.forEach((resolve) => resolve(null));
    this.captures.clear();
  }

  private post(request: TRendererRequest): void {
    this.worker.postMessage(request, listRendererTransfers(request));
  }
}
