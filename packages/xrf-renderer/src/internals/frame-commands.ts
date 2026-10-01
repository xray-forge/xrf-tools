import { Nullable, Optional } from "@xrf/types";
import { WebGPURenderer } from "three/webgpu";

import { RecordedPass } from "#/internals/recorded-pass";
import { getRendererBackend } from "#/internals/renderer-backend";
import { IStagedWrite } from "#/internals/staged-write";
import { StagedWrites } from "#/internals/staged-writes";
import { IStagingDevice } from "#/internals/staging-device";

/** A command encoder, as far as the frame shares one. */
interface IFrameEncoder {
  beginRenderPass(descriptor: unknown): object;
  beginComputePass(descriptor?: unknown): object;
  copyBufferToBuffer(source: object, sourceOffset: number, target: object, targetOffset: number, size: number): void;
  finish(descriptor?: unknown): unknown;
}

/** The queue, as far as the frame orders what reaches it. */
interface IFrameQueue {
  submit(buffers: Iterable<unknown>): void;
  writeBuffer(buffer: object, offset: number, data: BufferSource, dataOffset?: number, size?: number): void;
  writeTexture(...args: Array<unknown>): void;
  copyExternalImageToTexture(...args: Array<unknown>): void;
  onSubmittedWorkDone(): Promise<void>;
}

/** The device, as far as the frame hands out its encoders. */
interface IFrameDevice extends IStagingDevice {
  queue: IFrameQueue;
  createCommandEncoder(descriptor?: { label?: string }): IFrameEncoder;
}

type TQueueWrite = (buffer: object, offset: number, data: BufferSource, dataOffset?: number, size?: number) => void;

/** What three is handed for the shared encoder's command buffer: the frame submits the real one itself, once. */
const SHARED_COMMANDS: unique symbol = Symbol("shared frame commands");

/** Every device's frame commands, whose open frame a map of a buffer submits first: it waits on what was submitted. */
const adopted: Set<FrameCommands> = new Set();
let isMapGuarded: boolean = false;

/**
 * Records a frame's renders and computes into one command encoder, submitted once as the frame ends, where three makes
 * an encoder, a command buffer and a submit of each, every submit a cost of its own in the GPU process. Three writes
 * what a render reads as it records it, through the queue, which lands ahead of everything one submit holds: so each
 * pass is recorded and replayed into the encoder as it ends, and a buffer written once a pass of the frame was recorded
 * is written by a copy from a staging buffer, in front of the pass open as it is written or of the next one. Each pass
 * reads what it read with a submit of its own. A buffer mapped, a texture written, or a wait on the GPU first submits
 * what was recorded; an encoder three makes while a pass of the frame's is open is its own, as before. Outside a frame
 * every encoder is three's own.
 */
export class FrameCommands {
  /**
   * @param renderer - A renderer, its device open.
   * @returns Its frame commands, the device's encoders and queue routed through them.
   */
  public static adopt(renderer: WebGPURenderer): FrameCommands {
    return new FrameCommands(getRendererBackend(renderer).device as unknown as IFrameDevice);
  }

  /** Segments submitted: one a frame where nothing splits it. */
  public segments: number = 0;
  /** Writes made by a copy, in front of the pass reading them first. */
  public copies: number = 0;

  private readonly create: (descriptor?: { label?: string }) => IFrameEncoder;
  private readonly submitNow: (buffers: Iterable<unknown>) => void;
  private readonly write: TQueueWrite;
  private readonly whenDone: () => Promise<void>;
  private readonly staged: StagedWrites;
  private readonly pass: RecordedPass = new RecordedPass(() => this.endPass());
  /** The shared encoder of the segment recording, null before its first encoder is asked for. */
  private encoder: Nullable<IFrameEncoder> = null;
  private isOpen: boolean = false;
  /** The pass recording and how it begins. */
  private isPassOpen: boolean = false;
  private isRenderPass: boolean = false;
  private descriptor: unknown = null;
  /** The copies to make in front of the pass recording, five values each: staging buffer and offset, target, size. */
  private readonly pending: Array<unknown> = [];
  /** Passes the segment holds, ended: a write once one is, is made by a copy. */
  private passes: number = 0;
  /** Encoders three made of its own inside a pass of the frame's, not submitted yet: their writes are their own. */
  private nested: number = 0;

  private constructor(device: IFrameDevice) {
    const queue: IFrameQueue = device.queue;
    const writeTexture: (...args: Array<unknown>) => void = queue.writeTexture.bind(queue);
    const copyImage: (...args: Array<unknown>) => void = queue.copyExternalImageToTexture.bind(queue);

    this.create = device.createCommandEncoder.bind(device);
    this.submitNow = queue.submit.bind(queue);
    this.write = queue.writeBuffer.bind(queue);
    this.whenDone = queue.onSubmittedWorkDone.bind(queue);
    this.staged = new StagedWrites(device);

    device.createCommandEncoder = (descriptor?: { label?: string }): IFrameEncoder => this.toEncoder(descriptor);
    queue.submit = (buffers: Iterable<unknown>): void => this.submit(buffers);
    queue.writeBuffer = (
      buffer: object,
      offset: number,
      data: BufferSource,
      dataOffset?: number,
      size?: number
    ): void => this.writeBuffer(buffer, offset, data, dataOffset, size);
    // Rare in a frame, and a texture has no copy as cheap as a buffer's: what was recorded is submitted first.
    queue.writeTexture = (...args: Array<unknown>): void => {
      this.flushRecorded();
      writeTexture(...args);
    };
    queue.copyExternalImageToTexture = (...args: Array<unknown>): void => {
      this.flushRecorded();
      copyImage(...args);
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
    this.nested = 0;
  }

  /** Submits the segment recorded so far, the next encoder asked for starting another; never with a pass open. */
  public flush(): void {
    const { encoder } = this;

    if (!encoder || this.isPassOpen) {
      return;
    }

    this.encoder = null;
    this.passes = 0;
    this.segments += 1;
    // The staging buffers' bytes go ahead of the copies reading them, which the queue keeps in its order.
    this.staged.upload(this.write);
    this.submitNow([Object.getPrototypeOf(encoder).finish.call(encoder)]);
  }

  /** Lets the device go: its encoders and queue stay routed, but nothing is held for it. */
  public dispose(): void {
    this.end();
    this.staged.dispose();
    adopted.delete(this);
  }

  private toEncoder(descriptor?: { label?: string }): IFrameEncoder {
    // A pass of the frame's is open: three is making something of its own inside it, such as a texture's mipmaps.
    if (!this.isOpen) {
      return this.create(descriptor);
    }

    if (this.isPassOpen) {
      this.nested += 1;

      return this.create(descriptor);
    }

    if (!this.encoder) {
      const encoder: IFrameEncoder = this.create({ label: "frame" });

      encoder.beginRenderPass = (passDescriptor: unknown): object => this.beginPass(true, passDescriptor);
      encoder.beginComputePass = (passDescriptor?: unknown): object => this.beginPass(false, passDescriptor);
      encoder.finish = (): unknown => SHARED_COMMANDS;
      this.encoder = encoder;
    }

    return this.encoder;
  }

  private beginPass(isRender: boolean, descriptor: unknown): object {
    this.isPassOpen = true;
    this.isRenderPass = isRender;
    // Three resets the descriptor it shares between passes as soon as the pass begins, and the pass begins at its end.
    this.descriptor = toDescriptorCopy(descriptor);
    this.pass.reset();

    return this.pass;
  }

  /** Replays the pass into the encoder, the copies it reads first. */
  private endPass(): void {
    const encoder: IFrameEncoder = this.encoder as IFrameEncoder;
    const { pending } = this;

    for (let at: number = 0; at < pending.length; at += 5) {
      encoder.copyBufferToBuffer(
        pending[at] as object,
        pending[at + 1] as number,
        pending[at + 2] as object,
        pending[at + 3] as number,
        pending[at + 4] as number
      );
    }

    pending.length = 0;

    const prototype: IFrameEncoder = Object.getPrototypeOf(encoder);
    const real: object = this.isRenderPass
      ? prototype.beginRenderPass.call(encoder, this.descriptor)
      : prototype.beginComputePass.call(encoder, this.descriptor);

    this.pass.replay(real);
    (real as { end(): void }).end();
    this.descriptor = null;
    this.isPassOpen = false;
    this.passes += 1;
  }

  private writeBuffer(buffer: object, offset: number, data: BufferSource, dataOffset?: number, size?: number): void {
    // No pass recorded before the one it reaches: the queue's write lands in front of it as a submit of its own would.
    // Or made for an encoder of three's own, submitted at once, ahead of the frame.
    if (!this.isOpen || !this.encoder || this.passes === 0 || this.nested > 0) {
      this.write(buffer, offset, data, dataOffset, size);

      return;
    }

    const bytes: Uint8Array = toBytes(data, dataOffset, size);
    const at: IStagedWrite = this.staged.stage(bytes);

    this.copies += 1;

    if (this.isPassOpen) {
      this.pending.push(at.buffer, at.offset, buffer, offset, bytes.byteLength);
    } else {
      this.encoder.copyBufferToBuffer(at.buffer, at.offset, buffer, offset, bytes.byteLength);
    }
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
      this.nested = Math.max(0, this.nested - own.length);
      this.submitNow(own);
    }
  }

  /** Submits what was recorded where a pass of it was, for what the queue does at once to read it. */
  private flushRecorded(): void {
    if (this.passes > 0) {
      this.flush();
    }
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

/** A pass's descriptor, as far as three fills one it shares between passes and resets field by field. */
interface IPassDescriptor {
  colorAttachments?: ReadonlyArray<Nullable<object>>;
  depthStencilAttachment?: object;
  timestampWrites?: object;
}

/**
 * @returns A copy of a pass's descriptor as it stands, its attachments and timestamp writes copied with it: what three
 *   resets is their fields, never the views, query sets and clear values they name.
 */
function toDescriptorCopy(descriptor: unknown): unknown {
  if (!descriptor) {
    return descriptor;
  }

  const { colorAttachments, depthStencilAttachment, timestampWrites } = descriptor as IPassDescriptor;
  const copy: IPassDescriptor = { ...(descriptor as IPassDescriptor) };

  if (colorAttachments) {
    copy.colorAttachments = colorAttachments.map((attachment: Nullable<object>) => attachment && { ...attachment });
  }

  if (depthStencilAttachment) {
    copy.depthStencilAttachment = { ...depthStencilAttachment };
  }

  if (timestampWrites) {
    copy.timestampWrites = { ...timestampWrites };
  }

  return copy;
}

/**
 * @returns The bytes a queue write of these arguments sends: `dataOffset` and `size` count elements of a typed array
 *   and bytes of a buffer.
 */
function toBytes(data: BufferSource, dataOffset: number = 0, size?: number): Uint8Array {
  if (ArrayBuffer.isView(data)) {
    const element: number = (data as unknown as { BYTES_PER_ELEMENT?: number }).BYTES_PER_ELEMENT ?? 1;
    const count: number = size ?? data.byteLength / element - dataOffset;

    return new Uint8Array(data.buffer, data.byteOffset + dataOffset * element, count * element);
  }

  return new Uint8Array(data, dataOffset, size ?? data.byteLength - dataOffset);
}

declare const GPUBuffer: Optional<{ prototype: object }>;
