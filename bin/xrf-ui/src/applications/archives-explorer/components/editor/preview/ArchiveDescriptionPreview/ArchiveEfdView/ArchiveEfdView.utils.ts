import { ArchiveEfdDescription, ArchiveEfdPattern } from "@/core/ipc/types/xrf-app";
import { ABSENT_VALUE, formatNumber } from "@/lib/format/number";

import { formatCount } from "../ArchiveDescriptionPreview.utils";

/**
 * The band the function's result is trained between.
 *
 * @param description - Function to describe.
 * @returns The two ends, or the placeholder where the file declares neither.
 */
export function describeResultRange(description: ArchiveEfdDescription): string {
  const { minimumResult, maximumResult } = description;

  if (minimumResult === null && maximumResult === null) {
    return ABSENT_VALUE;
  }

  return `${formatNumber(minimumResult, 3)} to ${formatNumber(maximumResult, 3)}`;
}

/**
 * Which of the function's inputs one term reads.
 *
 * @param pattern - Term to describe.
 * @returns The inputs by their positions in the function's own list, or a phrase for a term reading none.
 */
export function describeTermInputs(pattern: ArchiveEfdPattern): string {
  return pattern.variables.length ? `Inputs ${pattern.variables.join(", ")}` : "Reads no input";
}

/**
 * How much of the weight table one term claims.
 *
 * @param pattern - Term to describe.
 * @returns The weight count with where it comes from, or a phrase for a term whose product cannot be taken.
 */
export function describeTermWeights(pattern: ArchiveEfdPattern): string {
  return pattern.weights === null
    ? "More weights than the ranges can be multiplied into"
    : `${formatCount(pattern.weights)} weights, the product of those inputs' ranges`;
}
