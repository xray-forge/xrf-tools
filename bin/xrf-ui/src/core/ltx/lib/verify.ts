import { XrfError } from "@/core/bindings/types/xrf-error";

/**
 * One scheme problem, unwrapped from the error variant that carries it.
 *
 * The backend reports failures as its own error enum, which serde tags externally: a scheme problem
 * arrives as `{ LtxScheme: { ... } }` rather than as its fields. Verification only ever produces that
 * variant, so this narrows to it and hands back the payload the table actually renders.
 */
export type TLtxSchemeError = Extract<XrfError, { LtxScheme: unknown }>["LtxScheme"];

/**
 * Extracts scheme problems from a verification result, discarding other error variants.
 *
 * @param errors - Verification errors to inspect.
 * @returns Scheme-problem payloads from the supplied errors.
 */
export function toLtxSchemeErrors(errors: Array<XrfError>): Array<TLtxSchemeError> {
  return errors
    .filter((it: XrfError): it is Extract<XrfError, { LtxScheme: unknown }> => "LtxScheme" in it)
    .map((it) => it.LtxScheme);
}
