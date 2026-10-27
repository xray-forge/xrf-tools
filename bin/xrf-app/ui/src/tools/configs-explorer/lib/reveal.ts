/**
 * What a panel asked the open view to bring into sight.
 */
export type TConfigsReveal =
  /** A section, wherever the open view draws it. */
  | { kind: "section"; section: string }
  /** One line of the authored config, which is where a finding is anchored. */
  | { kind: "line"; line: number };
