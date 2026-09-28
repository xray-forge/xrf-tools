/**
 * Fails to compile when a value the caller believed exhausted still has a case left.
 *
 * Reached only from the `default` of a switch that handles every member. There it exists for the compiler: an
 * enum-member `case` narrows a generated union but does not prove it exhausted - TypeScript does not accept a string
 * enum member as covering the string literal it equals - so a switch written with `EArchiveSubject.WORLD` instead of
 * `"world"` silently stops being checked. Handing the remainder to this restores the check, and the error names the
 * variant that was added.
 *
 * The throw is the runtime half of the same statement: a value arriving here came off IPC spelling something no
 * declaration holds, which is worth failing on rather than rendering as nothing.
 *
 * Declared as returning `never` so a call site may `return` it, leaving the switch with no unreachable fallback.
 *
 * @param value - The remainder of the switch, which is `never` when every case is handled.
 */
export function assertExhaustive(value: never): never {
  throw new TypeError(`Unhandled case: ${JSON.stringify(value)}`);
}
