import { Nullable } from "@xrf/types";

import { IStagedWrite } from "#/internals/staged-write";
import { IStagingDevice } from "#/internals/staging-device";

/** WebGPU's `GPUBufferUsage` flags of a staging buffer: written by the queue, copied from. */
const STAGING_USAGE: number = 0x04 | 0x08;
/** Bytes a staging chunk holds unless a write wants more: a frame's late uniform writes come to a few kilobytes. */
const CHUNK_BYTES: number = 1 << 16;

/** A staging buffer, as far as it is let go. */
type TStagingBuffer = ReturnType<IStagingDevice["createBuffer"]>;

/** A staging buffer, and its bytes on this side as the segment fills them. */
interface IStagingChunk {
  buffer: TStagingBuffer;
  bytes: Uint8Array<ArrayBuffer>;
  used: number;
}

/**
 * The bytes of the writes a segment orders by copies: each write's bytes copied into a staging chunk as it is made, and
 * every chunk used written to its buffer once, before the segment's submit, which the copies then read. A chunk is
 * written again only by a later segment's upload, which the queue orders after the copies of this one.
 */
export class StagedWrites {
  private readonly device: IStagingDevice;
  private readonly chunks: Array<IStagingChunk> = [];
  /** The chunk being filled, by its place in `chunks`; those before it are full. */
  private current: number = 0;
  private readonly answer: IStagedWrite = { buffer: null as unknown as object, offset: 0 };

  /**
   * @param device - What the staging buffers are made on.
   */
  public constructor(device: IStagingDevice) {
    this.device = device;
  }

  /**
   * @param bytes - A write's bytes, copied at once: the caller is free to change them after.
   * @returns Where they stand, an answer read at once and written again by the next write.
   */
  public stage(bytes: Uint8Array): IStagedWrite {
    let chunk: Nullable<IStagingChunk> = this.chunks[this.current] ?? null;

    if (chunk && chunk.used + bytes.byteLength > chunk.bytes.byteLength) {
      this.current += 1;
      chunk = this.chunks[this.current] ?? null;
    }

    // A chunk kept from an earlier segment too small for this write is replaced by one that holds it.
    if (!chunk || bytes.byteLength > chunk.bytes.byteLength) {
      // Its copies, if any, were all submitted with an earlier segment, which the GPU finishes before letting it go.
      chunk?.buffer.destroy();
      chunk = this.createChunk(Math.max(CHUNK_BYTES, bytes.byteLength));
      this.chunks[this.current] = chunk;
    }

    chunk.bytes.set(bytes, chunk.used);
    this.answer.buffer = chunk.buffer;
    this.answer.offset = chunk.used;
    // Copies want offsets of four bytes, which every write's size already is.
    chunk.used += (bytes.byteLength + 3) & ~3;

    return this.answer;
  }

  /**
   * Writes every chunk the segment used to its buffer, and starts the next segment empty.
   *
   * @param write - The queue's own write, ahead of the segment's submit.
   */
  public upload(
    write: (buffer: object, offset: number, data: Uint8Array<ArrayBuffer>, dataOffset: number, size: number) => void
  ): void {
    for (let at: number = 0; at <= this.current && at < this.chunks.length; at += 1) {
      const chunk: IStagingChunk = this.chunks[at];

      if (chunk.used) {
        write(chunk.buffer, 0, chunk.bytes, 0, chunk.used);
        chunk.used = 0;
      }
    }

    this.current = 0;
  }

  /** Lets every staging buffer go. */
  public dispose(): void {
    this.chunks.forEach((chunk: IStagingChunk) => chunk.buffer.destroy());
    this.chunks.length = 0;
    this.current = 0;
  }

  private createChunk(size: number): IStagingChunk {
    const bytes: number = (size + 3) & ~3;

    return {
      buffer: this.device.createBuffer({ label: "frame staging", size: bytes, usage: STAGING_USAGE }),
      bytes: new Uint8Array(bytes),
      used: 0,
    };
  }
}
