/** Whatever carries a cursor: an element on a page, or what stands in for one on a thread without a page. */
export interface ICursorTarget {
  style: { cursor: string };
}
