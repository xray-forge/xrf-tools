import {
  ArchiveDescriptor,
  ArchiveFileDescriptor,
  ArchiveProject,
  ArchiveSharedPayload,
} from "@/core/bindings/types/xrf-archive";
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

/**
 * The shared payload an entry is read from, if other entries read it too.
 *
 * Matched on the fields a reader locates a payload by, which is how the backend derived the group in the first place,
 * so an entry is found by what it reads rather than by its name.
 *
 * @param payloads - Shared payloads the backend derived for the open project.
 * @param descriptor - Entry to look up.
 * @returns The payload several entries read, or null when the entry has bytes of its own.
 */
export function findSharedPayloadOf(
  payloads: Array<ArchiveSharedPayload>,
  descriptor: Nullable<ArchiveFileDescriptor>
): Nullable<ArchiveSharedPayload> {
  if (!descriptor || descriptor.isDirectory) {
    return null;
  }

  return (
    payloads.find(
      (payload: ArchiveSharedPayload) =>
        payload.volume === descriptor.volume &&
        payload.offset === descriptor.offset &&
        payload.sizeCompressed === descriptor.sizeCompressed &&
        payload.sizeReal === descriptor.sizeReal &&
        payload.crc === descriptor.crc
    ) ?? null
  );
}

/**
 * Names of the other entries read from the same bytes as an entry.
 *
 * @param payloads - Shared payloads the backend derived for the open project.
 * @param descriptor - Entry whose sharers are wanted.
 * @returns The other names, in the order the backend sorted them; empty for a payload of its own.
 */
export function listPayloadSharersOf(
  payloads: Array<ArchiveSharedPayload>,
  descriptor: Nullable<ArchiveFileDescriptor>
): Array<string> {
  return findSharedPayloadOf(payloads, descriptor)?.names.filter((name: string) => name !== descriptor?.name) ?? [];
}
