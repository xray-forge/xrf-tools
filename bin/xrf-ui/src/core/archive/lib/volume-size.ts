import { Nullable } from "@xrf/types";

import { bytesToWholeMegabytes, megabytesToBytes } from "@/lib/memory/size";

export interface IResolvedArchiveVolumeSize {
  bytes: number;
  error: Nullable<string>;
  maxMegabytes: number;
}

/**
 * Resolves a per-run volume ceiling without changing the configured maximum.
 *
 * @param input - Ceiling in megabytes as typed in the form.
 * @param maxBytes - Configured maximum in bytes, or zero before defaults arrive.
 * @returns The displayed limit, input error, and effective ceiling; empty or invalid input uses the maximum.
 */
export function resolveArchiveVolumeSize(input: string, maxBytes: number): IResolvedArchiveVolumeSize {
  const maxMegabytes: number = bytesToWholeMegabytes(maxBytes);

  if (!input.trim()) {
    return { bytes: maxBytes, error: null, maxMegabytes };
  }

  const value: number = Number(input);

  if (!Number.isInteger(value) || value < 1 || value > maxMegabytes) {
    return {
      bytes: maxBytes,
      error: `Enter a whole number between 1 and ${maxMegabytes}`,
      maxMegabytes,
    };
  }

  return { bytes: megabytesToBytes(value), error: null, maxMegabytes };
}
