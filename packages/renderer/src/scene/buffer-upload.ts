import { BufferAttribute, InterleavedBuffer } from "three/webgpu";

/**
 * Queues a run of a buffer's elements to go up to the GPU with the buffer's next use. Three sends every range queued
 * since the last upload and clears them itself, so a range still queued is one not yet sent: this one joins it rather
 * than replacing it. Clearing it first lost whatever an earlier change queued for a buffer not used in between, as a
 * cascade drawn every fourth frame is not, and the GPU kept what the buffer held before.
 *
 * @param attribute - The buffer: an attribute, or the interleaved buffer several share.
 * @param start - The first element to send.
 * @param count - Elements to send from there.
 */
export function queueBufferUpload(attribute: BufferAttribute | InterleavedBuffer, start: number, count: number): void {
  const [queued] = attribute.updateRanges;

  if (queued) {
    const end: number = Math.max(queued.start + queued.count, start + count);

    attribute.clearUpdateRanges();
    start = Math.min(queued.start, start);
    count = end - start;
  }

  attribute.addUpdateRange(start, count);
  attribute.needsUpdate = true;
}
