/**
 * @param pool - Objects kept between frames, the first `count` of them this frame's.
 * @param count - How many are this frame's.
 * @param out - Where they are sorted, kept between frames.
 * @param order - What they are sorted by.
 * @returns `out`, holding this frame's in order.
 */
export function takeSorted<T>(
  pool: ReadonlyArray<T>,
  count: number,
  out: Array<T>,
  order: (a: T, b: T) => number
): Array<T> {
  out.length = count;

  for (let index: number = 0; index < count; index += 1) {
    out[index] = pool[index];
  }

  return out.sort(order);
}

/** Nearest first. */
export function byDistance<T extends { readonly distance: number }>(a: T, b: T): number {
  return a.distance - b.distance;
}
