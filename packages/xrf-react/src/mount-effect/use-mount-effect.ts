import { EffectCallback, useEffect, useRef } from "react";

/**
 * Runs an effect once when the component mounts.
 *
 * It is not re-run when its closure changes, so it reads everything as of mount: right for a one-off such as
 * prefilling a form, wrong for anything that has to stay current.
 *
 * @param effect - Effect to run on mount, optionally returning a cleanup to run on unmount.
 */
export function useMountEffect(effect: EffectCallback): void {
  const effectRef = useRef<EffectCallback>(effect);

  useEffect(() => effectRef.current(), []);
}
