import { BufferAttribute, InterleavedBuffer } from "three/webgpu";

/**
 * Queues a run of a buffer's elements to go up to the GPU with the buffer's next use. Three sends every range queued
 * since the last upload and clears them itself, so a range still queued is one not yet sent: this one joins every one
 * of them, whoever queued it, rather than replacing them. Clearing them first lost whatever an earlier change queued for
 * a buffer not used in between, as a cascade drawn every fourth frame is not, and the GPU kept what the buffer held.
 *
 * @param attribute - The buffer: an attribute, or the interleaved buffer several share.
 * @param start - The first element to send.
 * @param count - Elements to send from there.
 */
export function queueBufferUpload(attribute: BufferAttribute | InterleavedBuffer, start: number, count: number): void {
  let first: number = start;
  let end: number = start + count;

  for (const queued of attribute.updateRanges) {
    first = Math.min(first, queued.start);
    end = Math.max(end, queued.start + queued.count);
  }

  attribute.clearUpdateRanges();
  attribute.addUpdateRange(first, end - first);
  attribute.needsUpdate = true;
}
