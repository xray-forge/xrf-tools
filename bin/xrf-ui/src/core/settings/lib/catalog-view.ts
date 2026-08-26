import { Nullable } from "@/lib/types/general";

/**
 * Layouts the root catalog offers, in the order its switch presents them.
 */
export const CATALOG_VIEWS = ["grid", "rows"] as const;

export type TCatalogView = (typeof CATALOG_VIEWS)[number];

/** A first run looks like the card grid the catalog has always been. */
export const DEFAULT_CATALOG_VIEW: TCatalogView = "grid";

/**
 * Narrows a stored value to a view the launcher can render.
 *
 * @param value - Raw stored value, or `null` when absent.
 * @returns The matching view, or `DEFAULT_CATALOG_VIEW` when there is none.
 */
export function toCatalogView(value: Nullable<string>): TCatalogView {
  return CATALOG_VIEWS.find((view: TCatalogView) => view === value) ?? DEFAULT_CATALOG_VIEW;
}
