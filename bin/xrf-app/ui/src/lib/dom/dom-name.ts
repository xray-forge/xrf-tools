import { Maybe, Optional } from "@xrf/types";

/**
 * A test id from its fragments.
 *
 * @param values - Id fragments; `false`, `null`, `undefined`, and `""` are skipped.
 * @returns The truthy fragments joined by `-`, or undefined when the first is falsy.
 */
export function tid(...values: Array<Maybe<string | false>>): Optional<string> {
  return values[0] ? values.filter(Boolean).join("-") : undefined;
}

/**
 * A DOM id from its fragments, spelled as {@link tid} spells a test id.
 *
 * @param values - Id fragments; `false`, `null`, `undefined`, and `""` are skipped.
 * @returns The truthy fragments joined by `-`, or undefined when the first is falsy.
 */
export function uid(...values: Array<Maybe<string | false>>): Optional<string> {
  return tid(...values);
}
