import { Vector3d } from "@/core/ipc/types/xrf-db";
import { ABSENT_VALUE, formatNumber } from "@/lib/format/number";
import { Nullable } from "@/lib/types/general";

/**
 * Decimals a model coordinate is worth showing.
 *
 * Loose visuals are metres, and a weapon is a few centimetres across, so millimetres is the point where the numbers
 * stop telling a reader anything.
 */
const COORDINATE_DIGITS: number = 3;

/**
 * Formats one model coordinate.
 *
 * @param value - Coordinate to format, possibly absent.
 * @returns The coordinate at model precision, or a placeholder.
 */
export function formatCoordinate(value: Nullable<number>): string {
  return formatNumber(value, COORDINATE_DIGITS);
}

/**
 * Formats a coordinate triple.
 *
 * Lives here rather than in `lib/format` because it takes a generated binding type, and `lib/` may not import `core/`.
 *
 * @param vector - Vector to format, possibly absent.
 * @returns The three coordinates, or a placeholder.
 */
export function formatVector(vector: Nullable<Vector3d>): string {
  return vector
    ? `${formatCoordinate(vector.x)}, ${formatCoordinate(vector.y)}, ${formatCoordinate(vector.z)}`
    : ABSENT_VALUE;
}
