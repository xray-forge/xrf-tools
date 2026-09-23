import { Nullable } from "@xrf/types";
import { RenderTarget, Vector2, WebGPURenderer } from "three/webgpu";

import { BumpPlaneCapture } from "#/capture/bump-plane-capture";
import { unpadReadbackRows } from "#/capture/readback-rows";
import { ERendererCaptureSource, TRendererCaptureSource } from "#/contract/renderer-capture";
import { PresentPass } from "#/pass/present-pass";
import { RendererTextures } from "#/texture/renderer-textures";

/** A capture waiting for a frame to answer it. */
interface IPendingCapture {
  id: number;
  source: TRendererCaptureSource;
}

/** Hands a capture's picture back, or nothing where there was none to draw. */
type TCaptureReply = (id: number, image: Nullable<ImageBitmap>) => void;

/**
 * The captures asked for, each drawn into a target of its own and read back once what it shows can be drawn.
 */
export class RendererCaptures {
  private readonly present: PresentPass;
  private readonly bumpPlanes: BumpPlaneCapture;
  private readonly textures: RendererTextures;
  private readonly reply: TCaptureReply;
  private pending: Array<IPendingCapture> = [];
  /** Bumped by every cancel, so a read finishing late can tell its device went away. */
  private generation: number = 0;

  public constructor(present: PresentPass, textures: RendererTextures, reply: TCaptureReply) {
    this.present = present;
    this.textures = textures;
    this.bumpPlanes = new BumpPlaneCapture(textures);
    this.reply = reply;
  }

  public get hasPending(): boolean {
    return this.pending.length > 0;
  }

  /**
   * @param id - What the answer is sent under.
   * @param source - What to draw.
   */
  public push(id: number, source: TRendererCaptureSource): void {
    this.pending.push({ id, source });
  }

  /**
   * Draws every capture that can be drawn now and reads it back; the rest wait for a later frame.
   *
   * @param renderer - The renderer drawing.
   * @param hasView - Whether a view is attached, without which there is no frame to capture.
   * @param drawn - The drawing size of the frame just drawn, or null when that frame cannot be read: its targets were
   *   just allocated and read back cleared, or the scene is still settling.
   */
  public answer(renderer: WebGPURenderer, hasView: boolean, drawn: Nullable<Vector2>): void {
    const captures: ReadonlyArray<IPendingCapture> = this.pending;

    this.pending = [];

    for (const capture of captures) {
      const { id, source } = capture;

      if (source.kind === ERendererCaptureSource.FRAME) {
        if (!hasView) {
          this.reply(id, null);
        } else if (drawn) {
          this.read(renderer, id, drawn.x, drawn.y, (target) => this.present.draw(renderer, source.view, target));
        } else {
          this.pending.push(capture);
        }
      } else if (this.textures.isUploaded(source.bump) && this.textures.isUploaded(source.companion)) {
        this.read(renderer, id, source.width, source.height, (target) =>
          this.bumpPlanes.draw(renderer, source.plane, source.bump, source.companion, target)
        );
      } else {
        this.pending.push(capture);
      }
    }
  }

  /** Answers every waiting capture with nothing, and drops every read in flight, for a device going away. */
  public cancel(): void {
    this.generation += 1;
    this.pending.forEach(({ id }) => this.reply(id, null));
    this.pending = [];
  }

  public dispose(): void {
    this.cancel();
    this.bumpPlanes.dispose();
  }

  private read(
    renderer: WebGPURenderer,
    id: number,
    width: number,
    height: number,
    draw: (target: RenderTarget) => void
  ): void {
    if (width < 1 || height < 1) {
      this.reply(id, null);

      return;
    }

    const generation: number = this.generation;
    const target: RenderTarget = new RenderTarget(width, height, { depthBuffer: false });

    draw(target);

    renderer
      .readRenderTargetPixelsAsync(target, 0, 0, width, height)
      .then((pixels) => createImageBitmap(new ImageData(unpadReadbackRows(pixels as Uint8Array, width, height), width)))
      .then((image: ImageBitmap) => {
        if (generation === this.generation) {
          this.reply(id, image);
        } else {
          image.close();
        }
      })
      .catch(() => this.reply(id, null))
      .finally(() => target.dispose());
  }
}
