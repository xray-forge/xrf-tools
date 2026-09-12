import { IArchiveEntry } from "@/core/archive/entry";
import { listArchiveFiles } from "@/core/archive/files";
import { createArchiveRoots } from "@/core/archive/roots";
import { ArchiveSubject } from "@/core/bindings/types/xrf-app";
import { ArchiveReadPolicy } from "@/core/bindings/types/xrf-archive";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { Nullable } from "@/lib/types/general";

/**
 * Which of the two things the explorer has open.
 */
export const enum EArchiveSubject {
  /** A set of `.db` volumes, read as one name table. */
  VOLUMES = "volumes",
  /** A game folder, read as the engine mounts it: its archives and the loose tree in front of them. */
  WORLD = "world",
}

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
 * Engine paths a subject answers with more than one copy for.
 *
 * Always zero for a volume set: a name table keeps one entry per name, so the copies it merged away are gone before
 * anything can count them. Shadowing is a fact about mounts, which only a world has.
 *
 * @param subject - What the explorer has open, or null when nothing is.
 * @returns How many paths are held more than once.
 */
export function getSubjectShadowedCount(subject: Nullable<ArchiveSubject>): number {
  return subject?.kind === EArchiveSubject.WORLD ? subject.world.shadowedCount : 0;
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
 * What a subject may be read as, by extension and size.
 *
 * @param subject - What the explorer has open, or null when nothing is.
 * @returns The backend's read policy, or null when nothing is open.
 */
export function getSubjectReadPolicy(subject: Nullable<ArchiveSubject>): Nullable<ArchiveReadPolicy> {
  if (!subject) {
    return null;
  }

  return subject.kind === EArchiveSubject.WORLD ? subject.world.readPolicy : subject.project.readPolicy;
}

/**
 * The path a subject was opened at, for the toolbar to show when no file is selected.
 *
 * @param subject - What the explorer has open, or null when nothing is.
 * @returns The volume set's own root, the world's first root, or an empty string.
 */
export function getSubjectRoot(subject: Nullable<ArchiveSubject>): string {
  if (!subject) {
    return "";
  }

  return subject.kind === EArchiveSubject.WORLD ? (subject.world.roots.roots[0]?.path ?? "") : subject.project.root;
}

/**
 * The roots every asset read of a subject is addressed by.
 *
 * A world already carries the roots it was opened with, so a read answers for the arrangement the listing on screen
 * describes rather than for whatever the path means now.
 *
 * @param subject - What the explorer has open.
 * @returns The roots spec the backend can mount.
 */
export function getSubjectRoots(subject: ArchiveSubject): XrayRoots {
  return subject.kind === EArchiveSubject.WORLD ? subject.world.roots : createArchiveRoots(subject.project);
}
