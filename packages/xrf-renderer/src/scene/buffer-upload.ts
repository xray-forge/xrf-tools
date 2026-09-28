import { BufferAttribute, InterleavedBuffer } from "three/webgpu";

/** A run of a buffer's elements queued to go up, as three keeps them. */
type TUpdateRange = BufferAttribute["updateRanges"][number];

/**
 * Queues a run of a buffer's elements to go up with its next use. The runs already queued stay, whoever queued them: a
 * buffer not used in between, as a cascade drawn every fourth frame is not, would otherwise lose them. This run joins
 * those it overlaps or touches and no others: three leaves the runs in place where it creates a buffer, sending all of
 * it, and a span joining runs apart would send what lies between them again.
 *
 * @param attribute - The buffer: an attribute, or the interleaved buffer several share.
 * @param start - The first element to send.
 * @param count - Elements to send from there.
 */
export function queueBufferUpload(attribute: BufferAttribute | InterleavedBuffer, start: number, count: number): void {
  const apart: Array<TUpdateRange> = [];
  let first: number = start;
  let end: number = start + count;

  for (const queued of attribute.updateRanges) {
    if (queued.start <= end && first <= queued.start + queued.count) {
      first = Math.min(first, queued.start);
      end = Math.max(end, queued.start + queued.count);
    } else {
      apart.push(queued);
    }
  }

  attribute.clearUpdateRanges();
  apart.forEach((queued: TUpdateRange) => attribute.addUpdateRange(queued.start, queued.count));
  attribute.addUpdateRange(first, end - first);
  attribute.needsUpdate = true;
}
