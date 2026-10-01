import { Maybe, Nullable } from "@xrf/types";
import { WebGPURenderer } from "three/webgpu";

import { getRendererBackend } from "#/internals/renderer-backend";

/** A pass three records, as far as its end is watched. */
interface IFramePass {
  end(): void;
}

/** A command encoder, as far as the frame shares one. */
interface IFrameEncoder {
  beginRenderPass(descriptor: unknown): IFramePass;
  beginComputePass(descriptor?: unknown): IFramePass;
  finish(descriptor?: unknown): unknown;
}

/** The queue, as far as the frame orders what reaches it. */
interface IFrameQueue {
  submit(buffers: Iterable<unknown>): void;
  writeBuffer(buffer: object, ...rest: Array<unknown>): void;
  onSubmittedWorkDone(): Promise<void>;
}

/** The device, as far as the frame hands out its encoders. */
interface IFrameDevice {
  queue: IFrameQueue;
  createCommandEncoder(descriptor?: { label?: string }): IFrameEncoder;
}

/** What three is handed for the shared encoder's command buffer: the frame submits the real one itself, once. */
const SHARED_COMMANDS: unique symbol = Symbol("shared frame commands");

/** Every device's frame commands, whose open frame a map of a buffer submits first: it waits on what was submitted. */
const adopted: Set<FrameCommands> = new Set();
let isMapGuarded: boolean = false;

/**
 * Records a frame's renders and computes into one command encoder, submitted once as the frame ends, where three makes
 * an encoder, a command buffer and a submit of each: those are what the browser frees only at the worker's next major
 * collection, all at once, as the stall that follows it. Three writes the uniforms a render reads as it records it,
 * through the queue, and so ahead of everything one submit holds: the frame starts another encoder, a segment, where a
 * write would otherwise reach a render recorded earlier in it. A render context drawn again does (three shares its
 * camera's uniforms between its draws), and a buffer written again since a pass recorded does. A buffer read back, or a
 * wait on the GPU, first submits what was recorded. An encoder three makes while a pass of the frame's is open is its
 * own, as before. Outside a frame every encoder is three's own.
 */
export class FrameCommands {
  /**
   * @param renderer - A renderer, its device open.
   * @returns Its frame commands, the device's encoders and queue routed through them.
   */
  public static adopt(renderer: WebGPURenderer): FrameCommands {
    return new FrameCommands(getRendererBackend(renderer).device as unknown as IFrameDevice);
  }

  /** Segments submitted since the frames were counted last: one a frame where nothing splits them. */
  public segments: number = 0;
  /** Writes reaching a render recorded earlier in their segment, with a pass open, which nothing could order. */
  public conflicts: number = 0;

  private readonly create: (descriptor?: { label?: string }) => IFrameEncoder;
  private readonly submitNow: (buffers: Iterable<unknown>) => void;
  private readonly write: (buffer: object, ...rest: Array<unknown>) => void;
  private readonly whenDone: () => Promise<void>;
  /** The shared encoder of the segment recording, null before its first encoder is asked for. */
  private encoder: Nullable<IFrameEncoder> = null;
  private isOpen: boolean = false;
  private isPassOpen: boolean = false;
  /** Passes recorded in the segment, and each buffer written in it by the passes recorded before the write. */
  private passes: number = 0;
  private readonly written: Map<object, number> = new Map();
  /** The render contexts the segment draws, by three's encoder label for each. */
  private readonly contexts: Set<string> = new Set();
  /** Ends a pass of the shared encoder: one function, set on each pass, so nothing is made a pass. */
  private readonly endPass: (this: IFramePass) => void;
  private readonly beginRender: (this: IFrameEncoder, descriptor: unknown) => IFramePass;
  private readonly beginCompute: (this: IFrameEncoder, descriptor?: unknown) => IFramePass;

  private constructor(device: IFrameDevice) {
    const queue: IFrameQueue = device.queue;
    const watch = (pass: IFramePass): IFramePass => this.watch(pass);
    const close = (): void => {
      this.isPassOpen = false;
    };

    this.create = device.createCommandEncoder.bind(device);
    this.submitNow = queue.submit.bind(queue);
    this.write = queue.writeBuffer.bind(queue);
    this.whenDone = queue.onSubmittedWorkDone.bind(queue);
    // Each the prototype's own, a render pass's or a compute pass's, called on the instance they were set on.
    this.endPass = function (this: IFramePass): void {
      close();
      Object.getPrototypeOf(this).end.call(this);
    };
    this.beginRender = function (this: IFrameEncoder, descriptor: unknown): IFramePass {
      return watch(Object.getPrototypeOf(this).beginRenderPass.call(this, descriptor));
    };
    this.beginCompute = function (this: IFrameEncoder, descriptor?: unknown): IFramePass {
      return watch(Object.getPrototypeOf(this).beginComputePass.call(this, descriptor));
    };

    device.createCommandEncoder = (descriptor?: { label?: string }): IFrameEncoder => this.toEncoder(descriptor);
    queue.submit = (buffers: Iterable<unknown>): void => this.submit(buffers);
    queue.writeBuffer = (buffer: object, ...rest: Array<unknown>): void => {
      this.beforeWrite(buffer);
      this.write(buffer, ...rest);
    };
    queue.onSubmittedWorkDone = (): Promise<void> => {
      this.flush();

      return this.whenDone();
    };
    adopted.add(this);
    FrameCommands.guardMaps();
  }

  /** Starts a frame: every encoder three asks for until it ends is the frame's, but where a pass of it is open. */
  public begin(): void {
    this.flush();
    this.isOpen = true;
  }

  /** Ends the frame, submitting what it recorded. */
  public end(): void {
    this.flush();
    this.isOpen = false;
  }

  /** Lets the device go: its encoders and queue stay routed, but nothing is held for it. */
  public dispose(): void {
    this.end();
    adopted.delete(this);
  }

  /** Submits the segment recorded so far, the next encoder asked for starting another. */
  public flush(): void {
    const { encoder } = this;

    if (!encoder) {
      return;
    }

    this.encoder = null;
    this.passes = 0;
    this.written.clear();
    this.contexts.clear();
    this.segments += 1;
    this.submitNow([Object.getPrototypeOf(encoder).finish.call(encoder)]);
  }

  private toEncoder(descriptor?: { label?: string }): IFrameEncoder {
    // A pass of the frame's is open: three is making something of its own inside it, such as a texture's mipmaps.
    if (!this.isOpen || this.isPassOpen) {
      return this.create(descriptor);
    }

    const label: string = descriptor?.label ?? "";

    // Three shares a render context's camera uniforms between its renders, and writes them as each records.
    if (label.startsWith("renderContext_")) {
      if (this.contexts.has(label)) {
        this.flush();
      }

      this.contexts.add(label);
    }

    if (!this.encoder) {
      const encoder: IFrameEncoder = this.create({ label: "frame" });

      encoder.beginRenderPass = this.beginRender;
      encoder.beginComputePass = this.beginCompute;
      encoder.finish = (): unknown => SHARED_COMMANDS;
      this.encoder = encoder;
    }

    return this.encoder;
  }

  private watch(pass: IFramePass): IFramePass {
    this.isPassOpen = true;
    this.passes += 1;
    pass.end = this.endPass;

    return pass;
  }

  private submit(buffers: Iterable<unknown>): void {
    const own: Array<unknown> = [];

    for (const buffer of buffers) {
      if (buffer !== SHARED_COMMANDS) {
        own.push(buffer);
      }
    }

    // Made inside a pass of the frame's, or before the frame: three's own, submitted as it asks.
    if (own.length) {
      this.submitNow(own);
    }
  }

  private beforeWrite(buffer: object): void {
    if (!this.isOpen) {
      return;
    }

    const at: Maybe<number> = this.written.get(buffer);

    // Written already, and read since by a pass the segment holds: the earlier pass would read this write.
    if (at !== undefined && at < this.passes) {
      if (this.isPassOpen) {
        this.conflicts += 1;

        if (this.conflicts === 1) {
          console.error("A buffer was written again inside a pass of the frame: a pass recorded before reads it too.");
        }
      } else {
        this.flush();
      }
    }

    this.written.set(buffer, this.passes);
  }

  /** Has a buffer mapped while a frame records submit the frame's work first, which the map is to wait on. */
  private static guardMaps(): void {
    if (isMapGuarded || typeof GPUBuffer === "undefined") {
      return;
    }

    isMapGuarded = true;

    const prototype = GPUBuffer.prototype as unknown as { mapAsync(...args: Array<unknown>): Promise<void> };
    const original: (...args: Array<unknown>) => Promise<void> = prototype.mapAsync;

    prototype.mapAsync = function (this: unknown, ...args: Array<unknown>): Promise<void> {
      adopted.forEach((commands: FrameCommands) => commands.isOpen && commands.flush());

      return original.apply(this, args);
    };
  }
}

declare const GPUBuffer: { prototype: object } | undefined;
