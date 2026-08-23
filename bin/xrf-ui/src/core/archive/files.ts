import { ArchiveFileDescriptor, ArchiveProject } from "@/core/bindings/types/xrf-archive";
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
