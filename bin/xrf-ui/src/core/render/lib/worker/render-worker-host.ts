import { OffscreenRenderTarget } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import {
  ERenderFrame,
  ERenderFrameResponse,
  isRenderFrameRequest,
  TRenderFrameRequest,
  TRenderFrameResponse,
} from "@/core/render/lib/worker/render-frame-messages";
import { RenderProxyElement } from "@/core/render/lib/worker/render-proxy-element";
import { Logger } from "@/lib/logging";

/**
 * A viewport as the far side of a worker sees it: a vocabulary of its own, and a way to be let go.
 */
export interface IRenderWorkerScene<TRequest> {
  /**
   * Takes one of its own messages.
   *
   * @param request - What the page said about what is drawn.
   */
  take(request: TRequest): void;
  /** Releases the scene and everything it holds. */
  dispose(): void;
}

/**
 * Builds a viewport once there is a canvas to build it on.
 *
 * @param target - The transferred canvas, sized by whatever the page last measured.
 * @param element - What stands in for that canvas, for controls written against a page.
 * @returns The viewport, which from here is only its own vocabulary.
 */
export type TRenderWorkerSceneFactory<TRequest> = (
  target: OffscreenRenderTarget,
  element: RenderProxyElement
) => IRenderWorkerScene<TRequest>;

/** Where a served viewport sends what it has to say. */
export type TRenderWorkerReply<TResponse extends { kind: string }> = (response: TResponse) => void;

/**
 * Holds a canvas somebody else's thread handed over, and a viewport drawn on it.
 */
export class RenderWorkerHost<TRequest extends { kind: string }> {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly create: TRenderWorkerSceneFactory<TRequest>;
  private readonly reply: TRenderWorkerReply<TRenderFrameResponse>;

  private scene: Nullable<IRenderWorkerScene<TRequest>> = null;
  private target: Nullable<OffscreenRenderTarget> = null;
  private element: Nullable<RenderProxyElement> = null;

  public constructor(create: TRenderWorkerSceneFactory<TRequest>, reply: TRenderWorkerReply<TRenderFrameResponse>) {
    this.create = create;
    this.reply = reply;
  }

  /**
   * Takes one message, from either vocabulary.
   *
   * @param request - What the page said.
   */
  public take(request: TRequest | TRenderFrameRequest): void {
    if (!isRenderFrameRequest(request)) {
      this.scene?.take(request as TRequest);

      return;
    }

    switch (request.kind) {
      case ERenderFrame.START:
        return this.start(request);

      case ERenderFrame.RESIZE:
        this.target?.resize(request);
        this.element?.resize(request);

        return;

      case ERenderFrame.INPUT:
        return this.element?.dispatch(request.event);

      case ERenderFrame.DISPOSE:
        return this.dispose();
    }
  }

  private start(request: Extract<TRenderFrameRequest, { kind: ERenderFrame.START }>): void {
    this.dispose();

    this.log.info("Drawing on a canvas of", request.width, "x", request.height, "at", request.pixelRatio);

    this.target = new OffscreenRenderTarget(request.canvas, request);
    this.element = new RenderProxyElement(request, (cursor: string) =>
      this.reply({ cursor, kind: ERenderFrameResponse.CURSOR })
    );
    this.scene = this.create(this.target, this.element);
  }

  private dispose(): void {
    if (this.scene) {
      this.log.info("Releasing everything the viewport held");
    }

    this.scene?.dispose();
    this.scene = null;
    this.target = null;
    this.element = null;
  }
}
