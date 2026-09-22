import { IOffscreenRenderSize, IRenderInputEvent, RenderInputForwarder } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import {
  ERenderFrame,
  ERenderFrameResponse,
  listRenderFrameTransfers,
  TRenderFrameRequest,
} from "@/core/render/lib/worker/render-frame-messages";
import { Logger } from "@/lib/logging";

/** What a viewport's near half needs to hand its canvas over and keep talking to it. */
export interface IRenderWorkerClientOptions<TRequest extends { kind: string }, TResponse extends { kind: string }> {
  /** The worker, already constructed. */
  worker: Worker;
  /** The canvas on the page, whose drawing is handed over and whose input and layout are not. */
  target: DomRenderTarget;
  /** Whose name the failures are reported under. */
  log: Logger;
  /** Whether gestures on the canvas are forwarded, which a viewport with controls of its own does not want. */
  isInputForwarded?: boolean;
  /** What a viewport's own message carries that has to be moved rather than copied. */
  listTransfers?: (request: TRequest) => Array<Transferable>;
  /** Told everything the far side says that is about what is drawn rather than about the frame. */
  onResponse: (response: TResponse) => void;
}

/**
 * Hands a canvas to another thread, and keeps saying what has happened to it.
 */
export class RenderWorkerClient<TRequest extends { kind: string }, TResponse extends { kind: string }> {
  private readonly worker: Worker;
  private readonly target: DomRenderTarget;
  private readonly log: Logger;
  private readonly listTransfers: Nullable<(request: TRequest) => Array<Transferable>>;
  private readonly input: Nullable<RenderInputForwarder>;
  private readonly unobserve: () => void;

  public constructor({
    worker,
    target,
    log,
    isInputForwarded = true,
    listTransfers,
    onResponse,
  }: IRenderWorkerClientOptions<TRequest, TResponse>) {
    this.worker = worker;
    this.target = target;
    this.log = log;
    this.listTransfers = listTransfers ?? null;

    this.worker.onmessage = (event: MessageEvent<TResponse>): void => {
      // The cursor is the frame's answer rather than the scene's: three writes it on the element it was given,
      // and only this side has one to write it on.
      if (event.data.kind === ERenderFrameResponse.CURSOR) {
        this.input?.setCursor((event.data as unknown as { cursor: string }).cursor);

        return;
      }

      onResponse(event.data);
    };

    // A worker that dies on its way up says nothing at all otherwise: the page keeps posting into a thread that
    // will never answer, and the only symptom is a viewport that never draws a frame.
    this.worker.onerror = (event: ErrorEvent): void => this.log.error("The render worker failed:", event.message);

    this.log.info("Started a render worker");

    this.post({ canvas: target.canvas.transferControlToOffscreen(), kind: ERenderFrame.START, ...this.getSize() });

    this.input = isInputForwarded
      ? new RenderInputForwarder(target.canvas, (event: IRenderInputEvent) =>
          this.post({ event, kind: ERenderFrame.INPUT })
        )
      : null;
    this.unobserve = target.observe(() => this.post({ kind: ERenderFrame.RESIZE, ...this.getSize() }));
  }

  /**
   * Says one thing to the thread that draws.
   *
   * @param request - A message of the frame's, or one of the viewport's own.
   */
  public post(request: TRequest | TRenderFrameRequest): void {
    this.worker.postMessage(request, [
      ...listRenderFrameTransfers(request),
      ...(this.listTransfers?.(request as TRequest) ?? []),
    ]);
  }

  /** Stops the thread and gives the canvas back to the page. */
  public dispose(): void {
    this.unobserve();
    this.input?.dispose();

    // Terminating reclaims the thread whole, context and all; the message is what releases a server that is not
    // on a worker, and is said first so the two are released the same way.
    this.post({ kind: ERenderFrame.DISPOSE });
    this.worker.terminate();

    this.log.info("Terminated the render worker");
  }

  private getSize(): IOffscreenRenderSize {
    return { height: this.target.height, pixelRatio: this.target.pixelRatio, width: this.target.width };
  }
}
