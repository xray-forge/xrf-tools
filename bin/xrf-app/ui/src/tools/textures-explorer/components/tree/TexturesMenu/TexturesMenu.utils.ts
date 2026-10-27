/**
 * Why the tree is empty, which is three different situations and one message each.
 *
 * @param isLoading - Whether the listing is still arriving.
 * @param total - Textures the roots hold, before filtering.
 * @param filterCount - Filters currently narrowing the tree.
 * @returns What to say in place of the rows.
 */
export function describeEmptyTextureTree(isLoading: boolean, total: number, filterCount: number): string {
  if (isLoading) {
    return "Listing textures…";
  }

  return total && filterCount ? "No textures match the selected filters." : "No textures found under this root.";
}
