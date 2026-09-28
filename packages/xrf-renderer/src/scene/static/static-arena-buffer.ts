import { StorageBufferAttribute, StorageBufferNode, WebGPURenderer } from "three/webgpu";

import { copyStorageBuffers, createStorageBuffer, writeStorageBuffer } from "#/internals/storage-buffers";
import { IStorageCopy } from "#/internals/storage-copy";
import { createArenaNode } from "#/scene/static/static-arena-nodes.tsl";
import { StorageRetirement } from "#/uniforms/storage-retirement";

/** One run of words written into the buffer: where it starts, and the words. */
type TArenaWrite = readonly [number, Uint32Array];

/**
 * One of an arena's buffers, held on the GPU alone: grown there by copying it into a larger one, and written only where
 * geometry was placed. The CPU holds nothing of it but the words waiting to be sent with the next flush.
 */
export class StaticArenaBuffer {
  /** What every shader over the arena reads it through, pointed at each buffer that replaces it. */
  public readonly node: StorageBufferNode<"uint">;

  private readonly retirement: StorageRetirement;
  private attribute: StorageBufferAttribute;
  /** Words the buffer on the GPU holds, none before the first flush makes one. */
  private held: number = 0;
  /** Words it holds from the next flush. */
  private wanted: number = 0;
  /** The runs written since the last flush, in the order they were. */
  private readonly writes: Array<TArenaWrite> = [];

  /**
   * @param retirement - Where the buffers a growth replaces go.
   */
  public constructor(retirement: StorageRetirement) {
    this.retirement = retirement;
    // Bound by nothing that draws: every draw over the arena waits for a range placed, which a flush sends first.
    this.attribute = new StorageBufferAttribute(new Uint32Array(1), 1);
    this.node = createArenaNode(this.attribute);
  }

  /**
   * @param words - Words it has to hold from the next flush, a growth on the GPU where that is more than it holds.
   */
  public reserve(words: number): void {
    this.wanted = Math.max(this.wanted, words);
  }

  /**
   * @param start - The first word written, within what it holds from the next flush.
   * @param words - The words, sent with the next flush and held until then.
   */
  public write(start: number, words: Uint32Array): void {
    this.writes.push([start, words]);
  }

  /** The words waiting to be sent, which the CPU holds until the next flush. */
  public listPending(): Array<Uint32Array> {
    return this.writes.map(([, words]: TArenaWrite) => words);
  }

  /**
   * Grows the buffer on the GPU where it was asked to hold more, copying what it held into the larger one, then sends
   * every run written since, in order: each copy is submitted before the writes, which land in the larger buffer.
   *
   * @param renderer - A renderer, its device open; without one everything waits for a later flush.
   * @returns Whether the buffer was replaced, which whatever binds it records again for.
   */
  public flush(renderer: WebGPURenderer): boolean {
    let isReplaced: boolean = false;

    if (this.wanted > this.held) {
      const grown: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(1), 1);

      if (!createStorageBuffer(renderer, grown, this.wanted * Uint32Array.BYTES_PER_ELEMENT)) {
        return false;
      }

      if (this.held > 0) {
        const copy: IStorageCopy = {
          bytes: this.held * Uint32Array.BYTES_PER_ELEMENT,
          destination: grown,
          source: this.attribute,
        };

        if (copyStorageBuffers(renderer, [copy]).length) {
          console.error("An arena's buffer grew without what it held: the one it replaced is not on the GPU.");
        }
      }

      this.retirement.retire([this.attribute]);
      this.attribute = grown;
      this.node.value = grown;
      this.held = this.wanted;
      isReplaced = true;
    }

    for (const [start, words] of this.writes) {
      writeStorageBuffer(renderer, this.attribute, start * Uint32Array.BYTES_PER_ELEMENT, words);
    }

    this.writes.length = 0;

    return isReplaced;
  }

  /** Gives its buffer up, for it to go once nothing binds it. */
  public dispose(): void {
    this.retirement.retire([this.attribute]);
    this.writes.length = 0;
  }
}
