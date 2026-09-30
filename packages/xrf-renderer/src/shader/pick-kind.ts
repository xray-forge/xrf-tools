/**
 * What a texel of a pick names, in its first channel.
 */
export enum EPickKind {
  /** Nothing drawn there: the target as it was cleared. */
  NONE = 0,
  /** A static draw: its slot, then its place. */
  STATIC = 1,
  /** A part drawn plainly: its mesh's id, then the instance of it drawn. */
  PLAIN = 2,
}
