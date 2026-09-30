import { BYTES_PER_KILOBYTE } from "@/lib/memory/size";

const BYTE_UNITS = ["B", "KB", "MB", "GB", "TB"] as const;

/**
 * Formats a byte count with an adaptive binary unit.
 *
 * @param bytes - Byte count to format. Negative values are treated as zero.
 * @returns The byte count formatted with an adaptive binary unit.
 */
export function formatBytes(bytes: number): string {
  const safeBytes: number = Math.max(0, bytes);

  return formatBytesAs(safeBytes, byteUnitIndex(safeBytes));
}

/**
 * Formats a byte count as {@link formatBytes} does, its unit written against the number, as a status bar reads it.
 *
 * @param bytes - Byte count to format. Negative values are treated as zero.
 * @returns The byte count, such as `20MB`.
 */
export function formatBytesCompact(bytes: number): string {
  return formatBytes(bytes).replace(" ", "");
}

/**
 * Formats two byte counts that are meant to be compared, both in the larger value's unit.
 *
 * A pack summary reads `1.17 GB source, 0.78 GB written`: with each size picking its own unit the ratio
 * between them stops being visible at a glance.
 *
 * @param first - First byte count of the pair. Negative values are treated as zero.
 * @param second - Second byte count of the pair. Negative values are treated as zero.
 * @returns Both counts formatted in the unit of the larger one.
 */
export function formatBytesPair(first: number, second: number): [string, string] {
  const safeFirst: number = Math.max(0, first);
  const safeSecond: number = Math.max(0, second);
  const unitIndex: number = byteUnitIndex(Math.max(safeFirst, safeSecond));

  return [formatBytesAs(safeFirst, unitIndex), formatBytesAs(safeSecond, unitIndex)];
}

/** Index into {@link BYTE_UNITS} of the largest binary unit a byte count fills. */
function byteUnitIndex(bytes: number): number {
  if (bytes < BYTES_PER_KILOBYTE) {
    return 0;
  }

  return Math.min(Math.floor(Math.log(bytes) / Math.log(BYTES_PER_KILOBYTE)), BYTE_UNITS.length - 1);
}

function formatBytesAs(bytes: number, unitIndex: number): string {
  if (unitIndex === 0) {
    return `${bytes} B`;
  }

  const value: number = bytes / BYTES_PER_KILOBYTE ** unitIndex;
  const precision: number = value >= 100 ? 0 : value >= 10 ? 1 : 2;

  return `${Number(value.toFixed(precision))} ${BYTE_UNITS[unitIndex]}`;
}
