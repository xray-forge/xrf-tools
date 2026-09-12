/** Which rules resolve the tree: the two read the same files to different values. */
export const enum EConfigsDialect {
  LTX = "ltx",
  DLTX = "dltx",
}

export const DIALECT_IDS: ReadonlyArray<EConfigsDialect> = [EConfigsDialect.LTX, EConfigsDialect.DLTX];
