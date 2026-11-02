/** One labelled figure of a status segment's hover. */
export interface IShellStatusDetail {
  label: string;
  value: string;
  /** Indents the row under the one before it, as a part of that figure. */
  isNested?: boolean;
}
