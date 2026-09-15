import { ArchiveSpawnSection } from "@/core/ipc/types/xrf-app";

import { formatChunkId } from "../ArchiveDescriptionPreview.utils";

/** Below this share a section is not worth qualifying: the size beside it already says it is small. */
const NOTABLE_SHARE: number = 0.01;

/**
 * What a section is called, falling back to the id for one the format does not name.
 *
 * @param section - Section of the set.
 * @returns Its name, or the id as the number it is.
 */
export function formatSectionName(section: ArchiveSpawnSection): string {
  return section.label ?? formatChunkId(section.id);
}

/**
 * What share of the file a section occupies.
 *
 * The point of listing sections at all: vanilla's game graph is most of a 29 MB set, and knowing that is what tells a
 * reader why the file weighs what it does.
 *
 * @param section - Section of the set.
 * @param size - Bytes the whole file occupies.
 * @returns A phrase for its share, or null for one too small to be worth one.
 */
export function describeSectionWeight(section: ArchiveSpawnSection, size: number): string | null {
  if (!size) {
    return null;
  }

  const share: number = section.size / size;

  return share < NOTABLE_SHARE ? null : `${Math.round(share * 100)}% of the file`;
}
