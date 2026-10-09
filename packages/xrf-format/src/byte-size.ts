export const BYTES_PER_KILOBYTE: number = 1024;
export const BYTES_PER_MEGABYTE: number = 1024 * BYTES_PER_KILOBYTE;

/**
 * Convert a byte count to megabytes, unrounded.
 *
 * Rounding is left to the caller because the callers differ: a summary shows one decimal, a form field
 * wants a whole number.
 */
export function bytesToMegabytes(bytes: number): number {
  return bytes / BYTES_PER_MEGABYTE;
}

/** Convert a megabyte count to bytes, for the sizes that are typed in megabytes but stored in bytes. */
export function megabytesToBytes(megabytes: number): number {
  return megabytes * BYTES_PER_MEGABYTE;
}

/** Convert a byte count to whole megabytes, for the fields that hold no fraction. */
export function bytesToWholeMegabytes(bytes: number): number {
  return Math.round(bytesToMegabytes(bytes));
}
