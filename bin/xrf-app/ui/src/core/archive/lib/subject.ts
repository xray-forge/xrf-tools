import { ArchiveSubject, EArchiveSubject } from "@xrf/ipc/types/xrf-app-archives";
import { Nullable } from "@xrf/types";

import { IArchiveEntry } from "./entry";
import { listArchiveFiles } from "./files";

/**
 * How many sources a subject answers from: volumes for a volume set, mounts for a world.
 *
 * @param subject - What the explorer has open, or null when nothing is.
 * @returns The count, or zero when nothing is open.
 */
export function getSubjectSourceCount(subject: Nullable<ArchiveSubject>): number {
  if (!subject) {
    return 0;
  }

  return subject.kind === EArchiveSubject.WORLD ? subject.world.mounts.length : subject.project.archives.length;
}

/**
 * Unpacked bytes of what a subject holds. A world counts only the copies the engine would load.
 *
 * @param subject - What the explorer has open, or null when nothing is.
 * @returns The total, or zero when nothing is open.
 */
export function getSubjectSize(subject: Nullable<ArchiveSubject>): number {
  if (!subject) {
    return 0;
  }

  return subject.kind === EArchiveSubject.WORLD ? subject.world.sizeReal : subject.project.sizeReal;
}

/**
 * The files a subject holds, in the one shape every browsing surface reads.
 *
 * @param subject - What the explorer has open, or null when nothing is.
 * @returns Entries that are files, empty when nothing is open.
 */
export function listSubjectEntries(subject: Nullable<ArchiveSubject>): Array<IArchiveEntry> {
  if (!subject) {
    return [];
  }

  return subject.kind === EArchiveSubject.WORLD ? subject.world.files : listArchiveFiles(subject.project);
}

/**
 * The path a subject was opened at, for the toolbar to show when no file is selected.
 *
 * @param subject - What the explorer has open, or null when nothing is.
 * @returns The volume set's own root, the world's first root, or an empty string.
 */
export function getSubjectRoot(subject: Nullable<ArchiveSubject>): string {
  return subject?.roots.roots[0]?.path ?? "";
}
