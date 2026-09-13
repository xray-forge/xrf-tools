import { XrfApplicationError } from "@/core/error/lib";
import { Optional } from "@/lib/types/general";

/**
 * The member of `enumeration` that `value` spells.
 *
 * @param enumeration - A generated enum object, such as `ETextureCatalogMode`.
 * @param value - The spelling to narrow, as it crossed IPC.
 * @returns The member carrying that spelling.
 */
export function toEnumMember<T extends string>(enumeration: Record<string, T>, value: `${T}`): T {
  const member: Optional<T> = Object.values(enumeration).find((candidate) => candidate === value);

  if (member === undefined) {
    throw new XrfApplicationError(`Unknown enumeration member "${value}", which no generated declaration holds.`);
  }

  return member;
}
