import { BufferAttribute } from "three/webgpu";

import { queueBufferUpload } from "#/scene/buffer-upload";

/**
 * The elements of a buffer written since it last went up, as one span, uploaded whole.
 */
export class DirtySpan {
  private first: number = Infinity;
  private last: number = -1;

  /** Whether nothing was written since the span was last cleared. */
  public get isEmpty(): boolean {
    return this.last < this.first;
  }

  /**
   * @param first - The first element written.
   * @param last - The last element written, the first where only it was; before it where none was.
   */
  public touch(first: number, last: number = first): void {
    if (last < first) {
      return;
    }

    this.first = Math.min(this.first, first);
    this.last = Math.max(this.last, last);
  }

  /**
   * Queues the span's elements of an attribute for upload, where anything was written.
   *
   * @param attribute - A buffer the span's elements are in.
   * @param stride - Values one element takes in it.
   */
  public upload(attribute: BufferAttribute, stride: number): void {
    if (!this.isEmpty) {
      queueBufferUpload(attribute, this.first * stride, (this.last - this.first + 1) * stride);
    }
  }

  public clear(): void {
    this.first = Infinity;
    this.last = -1;
  }
}
