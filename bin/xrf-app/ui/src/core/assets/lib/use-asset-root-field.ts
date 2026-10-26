import { EApplicationId } from "@/core/routing/application";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";

/**
 * Names the field in every surface that offers one, so the four of them cannot drift apart.
 *
 * Per application rather than shared: a surface resolves against what it was told, and one value four tools read
 * silently is the ambient setting this replaced.
 */
const FIELD_ID: string = "asset-root";

/**
 * The extra tree this surface searches behind the one it opened.
 *
 * Optional, because reading only what you opened is a legitimate answer and the common one for a complete game tree.
 *
 * @param application - Application the field belongs to, which is also what scopes its history.
 * @param isDisabled - Whether selection is disabled, usually while the surface is busy.
 * @returns The field, with its own remembered value and its own history.
 */
export function useAssetRootField(application: EApplicationId, isDisabled?: boolean): IPathField {
  return usePathField({
    application,
    id: FIELD_ID,
    isDirectory: true,
    isDisabled,
    isRequired: false,
    title: "Select a game installation or game data directory",
  });
}
