import { IEquipmentSectionDescriptor } from "@/core/sprite-equipment/lib";

/**
 * One inventory icon rectangle, measured in grid cells.
 *
 * @param section - Section owning the rectangle.
 * @param overrides - Position and size to place it at; a one-cell slot at the origin by default.
 * @returns The rectangle as the configuration would declare it.
 */
export function mockEquipmentDescriptor(
  section: string,
  overrides: Partial<Omit<IEquipmentSectionDescriptor, "section">> = {}
): IEquipmentSectionDescriptor {
  return { section, x: 0, y: 0, w: 1, h: 1, ...overrides };
}
