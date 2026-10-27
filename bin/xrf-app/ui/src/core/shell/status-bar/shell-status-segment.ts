import { IShellStatusDetail } from "@/core/shell/status-bar/shell-status-detail";

/** A status bar segment with more to say than fits the bar, shown on hover. */
export interface IShellStatusSegment {
  /** Names the segment across publications, so its hover stays open while its figures change. */
  id: string;
  text: string;
  /** What the hover shows, one figure to a row. */
  details: ReadonlyArray<IShellStatusDetail>;
}

/** What an editor publishes to the status bar: plain text, or text with its details behind it. */
export type TShellStatusSegment = string | IShellStatusSegment;

/**
 * Tells whether two segments say the same thing, which is what keeps a republished status from redrawing the bar.
 *
 * @param first - One segment.
 * @param second - The other.
 * @returns Whether both are the same segment with the same text and the same details.
 */
export function isSameStatusSegment(first: TShellStatusSegment, second: TShellStatusSegment): boolean {
  if (typeof first === "string" || typeof second === "string") {
    return first === second;
  }

  return (
    first.id === second.id &&
    first.text === second.text &&
    first.details.length === second.details.length &&
    first.details.every((detail: IShellStatusDetail, index: number) => isSameDetail(detail, second.details[index]))
  );
}

function isSameDetail(first: IShellStatusDetail, second: IShellStatusDetail): boolean {
  return (
    first.label === second.label &&
    first.value === second.value &&
    (first.isNested ?? false) === (second.isNested ?? false)
  );
}
