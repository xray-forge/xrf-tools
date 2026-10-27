import { transformError } from "@xrf/errors";

/**
 * @param what - What failed, as a sentence goes on from "Failed to": `take a screenshot`, `export the view`.
 * @param error - Why.
 * @returns What the level viewer says of something done with its files that failed, every one alike.
 */
export function describeLevelViewerFailure(what: string, error: unknown): string {
  return `Failed to ${what}: ${transformError(error).message}`;
}
