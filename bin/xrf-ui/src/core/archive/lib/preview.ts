import { IArchiveEntry } from "@/core/archive/lib/entry";
import { ArchiveReadPolicy } from "@/core/ipc/types/xrf-archive";
import { EXrayExtension, XrayExtension } from "@/core/ipc/types/xrf-extension";
import { getFoldedFileExtension } from "@/lib/path/extension";

/** A preview representation, or the policy reason an entry cannot be previewed. */
export type ArchivePreviewSupport =
  | { kind: "supported" }
  | { kind: "image" }
  | { kind: "audio" }
  | { kind: "model" }
  | { kind: "description" }
  | { kind: "too-large"; maximumSize: number };

/**
 * Classifies an entry's preview using the backend-provided extensions and size limits.
 *
 * @param descriptor - Browsed entry used to validate type and size.
 * @param policy - Backend-provided archive read capabilities.
 * @returns A discriminated result describing preview support or the reason it is unavailable.
 */
export function getArchivePreviewSupport(descriptor: IArchiveEntry, policy: ArchiveReadPolicy): ArchivePreviewSupport {
  // Policy extensions are lower case; normalize the entry's spelling once for every preview kind.
  const extension: string = getFoldedFileExtension(descriptor.name);

  // Models are read through the asset roots rather than through this project, so no policy limit applies to them.
  if (extension === EXrayExtension.OGF) {
    return { kind: "model" };
  }

  // Media reads have their own limits. Stored compression is handled by the native readers.
  if (policy.audioExtensions.some((candidate: XrayExtension) => candidate === extension)) {
    return descriptor.sizeReal > policy.maximumAudioSize
      ? { kind: "too-large", maximumSize: policy.maximumAudioSize }
      : { kind: "audio" };
  }

  if (policy.imageExtensions.some((candidate: XrayExtension) => candidate === extension)) {
    return descriptor.sizeReal > policy.maximumImageSize
      ? { kind: "too-large", maximumSize: policy.maximumImageSize }
      : { kind: "image" };
  }

  if (!policy.extensions.some((candidate: XrayExtension) => candidate === extension)) {
    return { kind: "description" };
  }

  if (descriptor.sizeReal > policy.maximumSize) {
    return { kind: "too-large", maximumSize: policy.maximumSize };
  }

  return { kind: "supported" };
}
