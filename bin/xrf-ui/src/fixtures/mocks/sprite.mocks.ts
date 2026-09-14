import { EEquipmentSlotClaim, EquipmentSlotOccupant } from "@/core/ipc/types/xrf-texture";

/**
 * One section occupying a slot, measured in grid cells.
 *
 * Probable by default, because that is what every shipped tree answers: no gamedata in the workspace declares
 * `$inventory_icon` at all, so a fixture defaulting to declared would be the unrepresentative case.
 *
 * @param section - Section owning the slot.
 * @param overrides - What this case varies; a one-cell slot at the origin otherwise.
 * @returns The occupant as the backend would report it.
 */
export function mockEquipmentOccupant(
  section: string,
  overrides: Partial<Omit<EquipmentSlotOccupant, "section">> = {}
): EquipmentSlotOccupant {
  return {
    section,
    claim: EEquipmentSlotClaim.PROBABLE,
    customIcon: null,
    origin: null,
    x: 0,
    y: 0,
    w: 1,
    h: 1,
    ...overrides,
  };
}
