/** How an attribute's components sit in the words a clustered draw reads its vertices from. */
export enum EClusterWordFormat {
  /** A float a word, its bits. */
  FLOAT = "float",
  /** An unsigned integer a word. */
  UINT = "uint",
  /** Four normalized bytes, one word. */
  UNORM8X4 = "unorm8x4",
}
