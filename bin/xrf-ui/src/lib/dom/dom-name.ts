import { Maybe, Optional } from "@/lib/types/general";

/**
 * @param values - Id fragments; `false`, `null`, `undefined`, and `"` are skipped.
 * @returns The truthy fragments joined by a single space.
 */
export function tid(...values: Array<Maybe<string | false>>): Optional<string> {
  return values[0] ? values.filter(Boolean).join("-") : undefined;
}

/**
 * @param values - Id fragments; `false`, `null`, `undefined`, and `"` are skipped.
 * @returns The truthy fragments joined by a single space.
 */
export function uid(...values: Array<Maybe<string | false>>): Optional<string> {
  return values[0] ? values.filter(Boolean).join("-") : undefined;
}

/**
 * @param values - Class name fragments; `false`, `null`, `undefined`, and `"` are skipped.
 * @returns The truthy fragments joined by a single space.
 */
export function cls(...values: Array<Maybe<string | false>>): string {
  return values.filter(Boolean).join(" ");
}
