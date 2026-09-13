import { useCallback, useState } from "react";

/**
 * Renders the component again on demand.
 *
 * @returns A callback that schedules one re-render.
 */
export function useForceUpdate(): () => void {
  const [, setRevision] = useState<number>(0);

  return useCallback(() => setRevision((it: number) => it + 1), []);
}
