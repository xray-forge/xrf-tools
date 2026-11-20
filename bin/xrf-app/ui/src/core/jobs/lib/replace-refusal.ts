import { Nullable } from "@xrf/types";

/** How every refusal of `xrf-utils`' `OutputFiles` ends: the files a run writes are there, and force replaces them. */
const REPLACE_REFUSAL: string = "force it to replace them";

/**
 * @param error - Why a run failed, or null where it did not.
 * @returns Whether an unforced run refused only because files it writes are already there, which a forced run replaces.
 */
export function isReplaceRefusal(error: Nullable<Error>): boolean {
  return error !== null && error.message.includes(REPLACE_REFUSAL);
}
