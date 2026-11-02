/** One path a field was given, and when. */
export interface IPathRecord {
  path: string;
  /** Milliseconds since the epoch, for saying how old the entry is. Order is carried by position, not by this. */
  at: number;
}

/**
 * A field's history as the surfaces that offer it see it.
 */
export interface IPathFieldRecents {
  /** The remembered paths, newest first. */
  records: ReadonlyArray<IPathRecord>;
  /** Takes one of them. */
  pick: (path: string) => void;
  /** Drops one of them. */
  forget: (path: string) => void;
}
