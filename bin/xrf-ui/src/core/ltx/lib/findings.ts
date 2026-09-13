import { LtxAnchoredFinding, LtxFindingKind } from "@/core/ipc/types/xrf-ltx-inspect";
import { ECodeLineMark } from "@/core/ui/code/code-line";

/**
 * How each kind of finding is drawn beside the line that raised it.
 */
const FINDING_MARKS: Record<LtxFindingKind, ECodeLineMark> = {
  dialect: ECodeLineMark.WARNING,
  include: ECodeLineMark.ERROR,
  parse: ECodeLineMark.ERROR,
  scheme: ECodeLineMark.ERROR,
};

/**
 * The mark one finding puts in the gutter.
 *
 * @param finding - The finding to draw.
 * @returns The mark for its kind.
 */
export function toFindingMark(finding: LtxAnchoredFinding): ECodeLineMark {
  return FINDING_MARKS[finding.kind];
}

/**
 * The findings that belong to one config, by the line each one sits on.
 *
 * A line can raise several - a section breaks its scheme in three ways - and the gutter has room for one mark, so the
 * worst wins. Findings the anchor could not place on a line are dropped here and kept by the Problems panel, which is
 * the surface that can show a finding without a line to point at.
 *
 * @param findings - Findings of the root and of the file, in any order.
 * @param path - Engine identity of the config being drawn.
 * @returns The mark for each line that has one.
 */
export function toFindingMarks(
  findings: ReadonlyArray<LtxAnchoredFinding>,
  path: string
): ReadonlyMap<number, ECodeLineMark> {
  const marks: Map<number, ECodeLineMark> = new Map();

  for (const finding of findings) {
    if (finding.file !== path || finding.line === null) {
      continue;
    }

    const mark: ECodeLineMark = toFindingMark(finding);

    if (mark === ECodeLineMark.ERROR || !marks.has(finding.line)) {
      marks.set(finding.line, mark);
    }
  }

  return marks;
}

/**
 * Findings in the order a person reads them: by config, then down the file.
 *
 * Not by severity. A reader opening the panel is looking for a place to go, and a list that jumps between files to put
 * the errors first makes the same file appear twice.
 *
 * @param findings - Findings to order.
 * @returns The same findings, ordered.
 */
export function toOrderedFindings(findings: ReadonlyArray<LtxAnchoredFinding>): Array<LtxAnchoredFinding> {
  return [...findings].sort((first: LtxAnchoredFinding, second: LtxAnchoredFinding) => {
    const byFile: number = (first.file ?? "").localeCompare(second.file ?? "");

    return byFile === 0 ? (first.line ?? 0) - (second.line ?? 0) : byFile;
  });
}
