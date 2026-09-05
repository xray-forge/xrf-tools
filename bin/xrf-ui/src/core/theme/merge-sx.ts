import { SxProps, Theme } from "@mui/material/styles";

import { Optional } from "@/lib/types/general";

type SxArray = Extract<SxProps<Theme>, ReadonlyArray<unknown>>;

/**
 * Composes styles in order, with later entries overriding earlier ones.
 *
 * Flattens `sx` arrays without evaluating callbacks or merging style objects.
 *
 * @param styles - Style entries in override order; `undefined` entries are omitted.
 * @returns A flat style array for MUI to resolve.
 */
export function mergeSx(...styles: Array<Optional<SxProps<Theme>>>): SxProps<Theme> {
  return styles.flatMap<SxArray[number]>((sx) => (sx === undefined ? [] : sx));
}
