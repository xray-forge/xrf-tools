import { ArchiveDescriptor, ArchiveFileDescriptor, ArchiveProject } from "@/core/bindings/types/xrf-archive";
import { Nullable } from "@/lib/types/general";

/**
 * The files an archive project holds, without the directories its volumes record.
 *
 * A name table lists the directories a volume contains beside its files, and `isDirectory` marks which is which. Read
 * the set through here rather than through `project.files` directly: every surface that counted or listed the raw map
 * showed the directory entries as files, once as a phantom leaf beside the directory of the same name and once in the
 * file total.
 *
 * @param project - Opened archive project, or null when none is open.
 * @returns Descriptors of the entries that are files.
 */
export function listArchiveFiles(project: Nullable<ArchiveProject>): Array<ArchiveFileDescriptor> {
  return project
    ? Object.values(project.files).filter((descriptor: ArchiveFileDescriptor) => !descriptor.isDirectory)
    : [];
}

/**
 * The volume an entry's payload sits in.
 *
 * An entry records its volume as a position in the project's `archives`, so the set it belongs to is what turns that
 * back into a path. Reading it through here keeps the position an implementation detail of the model rather than
 * something every view indexes for itself.
 *
 * @param project - Opened archive project, or null when none is open.
 * @param descriptor - Entry whose volume is wanted.
 * @returns The volume holding the entry, or null when the project does not hold that position.
 */
export function getArchiveVolumeOf(
  project: Nullable<ArchiveProject>,
  descriptor: Nullable<ArchiveFileDescriptor>
): Nullable<ArchiveDescriptor> {
  return project && descriptor ? (project.archives[descriptor.volume] ?? null) : null;
}
