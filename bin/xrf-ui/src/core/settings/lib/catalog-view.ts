import { Nullable } from "@/lib/types/general";

/**
 * Layouts the root catalog offers, in the order its switch presents them.
 */
export const CATALOG_VIEWS = ["rows", "grid"] as const;

export type TCatalogView = (typeof CATALOG_VIEWS)[number];

/** A first run reads the whole catalog at once, which only the dense rows fit; cards are asked for. */
export const DEFAULT_CATALOG_VIEW: TCatalogView = "rows";

/**
 * Narrows a stored value to a view the launcher can render.
 *
 * @param value - Raw stored value, or `null` when absent.
 * @returns The matching view, or `DEFAULT_CATALOG_VIEW` when there is none.
 */
export function toCatalogView(value: Nullable<string>): TCatalogView {
  return CATALOG_VIEWS.find((view: TCatalogView) => view === value) ?? DEFAULT_CATALOG_VIEW;
}
