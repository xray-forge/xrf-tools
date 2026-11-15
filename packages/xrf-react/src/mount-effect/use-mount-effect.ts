import { EffectCallback, useEffect, useRef } from "react";

/**
 * Runs an effect once when the component mounts.
 *
 * The effect is not re-run when its closure changes, so anything it reads is read as of mount. That is
 * the point for a one-off - prefilling a form, kicking off a load - and the wrong tool for anything that
 * has to stay current.
 *
 * @param effect - Effect to run on mount, optionally returning a cleanup to run on unmount.
 */
export function useMountEffect(effect: EffectCallback): void {
  const effectRef = useRef<EffectCallback>(effect);

  useEffect(() => effectRef.current(), []);
}
