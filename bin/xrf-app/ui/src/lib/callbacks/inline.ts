/**
 * Evaluates a callback immediately so a multi-branch expression can stay local.
 *
 * @param callback - Function that computes the expression.
 * @returns The value the callback computes.
 */
export function inline<T = void>(callback: () => T): T {
  return callback();
}
