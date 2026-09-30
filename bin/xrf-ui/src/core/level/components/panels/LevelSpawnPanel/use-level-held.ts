import { Nullable } from "@xrf/types";
import { useEffect, useState } from "react";

import { ILevelHeldSource } from "@/core/level/lib/render/level-render-protocol";

/**
 * @param source - Something the level holds, told as it is now and whenever it changes.
 * @returns What it holds now, redrawing whenever that changes.
 */
export function useLevelHeld<T>(source: ILevelHeldSource<T>): Nullable<T> {
  const [value, setValue] = useState<Nullable<T>>(null);

  useEffect(() => source.subscribe((next: Nullable<T>) => setValue(() => next)), [source]);

  return value;
}
