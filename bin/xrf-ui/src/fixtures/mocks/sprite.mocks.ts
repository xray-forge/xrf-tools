import { EquipmentSheetLocation, EquipmentSpriteMetadata, EquipmentSpriteOpen } from "@/core/ipc/types/xrf-app";
import { EEquipmentSlotClaim, EquipmentSlotOccupant } from "@/core/ipc/types/xrf-texture";
import { Nullable } from "@/lib/types/general";

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

/**
 * An open naming loose files, which is the only shape a repack can act on.
 *
 * @param overrides - What this case varies.
 * @returns The request as the form would build it.
 */
export function mockEquipmentSpriteOpen(overrides: Partial<EquipmentSpriteOpen> = {}): EquipmentSpriteOpen {
  return {
    roots: { asset: null, roots: [] },
    sheet: { kind: "file", path: "C:\\game\\equipment.dds" },
    config: { kind: "file", path: "C:\\game\\system.ltx" },
    isDltx: false,
    ...overrides,
  };
}

/**
 * Where the sheet an open names turned out to be.
 *
 * @param open - The open whose sheet this locates.
 * @returns The location as the backend would report it.
 */
export function mockEquipmentSheetLocation(open: EquipmentSpriteOpen): EquipmentSheetLocation {
  const path: Nullable<string> = open.sheet.kind === "file" ? open.sheet.path : null;

  return { asset: null, path, writeTarget: path };
}

/**
 * What the backend answers for one opened sheet.
 *
 * @param overrides - What this case varies.
 * @returns The metadata as it crosses IPC.
 */
export function mockEquipmentSpriteMetadata(overrides: Partial<EquipmentSpriteMetadata> = {}): EquipmentSpriteMetadata {
  const open: EquipmentSpriteOpen = overrides.open ?? mockEquipmentSpriteOpen();

  return {
    name: "equipment.png",
    open,
    location: mockEquipmentSheetLocation(open),
    configError: null,
    occupants: [],
    ...overrides,
  };
}
