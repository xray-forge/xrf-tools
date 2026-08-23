import { ArchiveFileDescriptor, ArchiveProjectReadPolicy } from "@/core/bindings/types/xrf-archive";

export type ArchivePreviewSupport =
  | { kind: "supported" }
  | { kind: "image" }
  | { kind: "audio" }
  | { kind: "model" }
  | { kind: "unsupported-extension"; extension: string }
  | { kind: "too-large"; maximumSize: number };

/**
 * Checks whether an archive entry is a model the viewer can render.
 *
 * @param descriptor - Archive file metadata whose extension is checked.
 * @returns Whether the descriptor names a model.
 */
export function isArchiveModel(descriptor: ArchiveFileDescriptor): boolean {
  return descriptor.extension.toLowerCase() === "ogf";
}

/**
 * Checks whether the policy permits audio preview for an archive file.
 *
 * @param descriptor - Archive file metadata whose extension is checked.
 * @param policy - Backend-provided archive read policy.
 * @returns Whether the descriptor extension supports audio preview.
 */
export function isArchiveAudio(descriptor: ArchiveFileDescriptor, policy: ArchiveProjectReadPolicy): boolean {
  const extension: string = descriptor.extension.toLowerCase();

  return policy.audioExtensions.some((candidate: string) => candidate.toLowerCase() === extension);
}

/**
 * Checks whether the backend decodes an archive file as an image rather than reads it as text.
 *
 * Both lists come from the project's own read policy, so the frontend never has to keep its own copy of
 * what the backend is willing to do.
 *
 * @param descriptor - Archive file metadata whose extension is checked.
 * @param policy - Backend-provided archive read policy.
 * @returns Whether the descriptor extension supports image preview.
 */
export function isArchiveImage(descriptor: ArchiveFileDescriptor, policy: ArchiveProjectReadPolicy): boolean {
  const extension: string = descriptor.extension.toLowerCase();

  return policy.imageExtensions.some((candidate: string) => candidate.toLowerCase() === extension);
}

/**
 * Determine whether the backend can provide a text preview for an archive file.
 *
 * @param descriptor - Archive file metadata used to validate type, size, and compression state.
 * @param policy - Backend-provided archive read capabilities.
 * @returns A discriminated result describing preview support or the reason it is unavailable.
 */
export function getArchivePreviewSupport(
  descriptor: ArchiveFileDescriptor,
  policy: ArchiveProjectReadPolicy
): ArchivePreviewSupport {
  // Models are read through the asset roots rather than through this project, so no policy limit applies to them.
  if (isArchiveModel(descriptor)) {
    return { kind: "model" };
  }

  // Images are decoded rather than read as text, so they answer to their own limit and - unlike text -
  // do not care whether the entry was stored compressed. Decompression happens on the way out anyway.
  if (isArchiveAudio(descriptor, policy)) {
    return descriptor.sizeReal > policy.maximumAudioSize
      ? { kind: "too-large", maximumSize: policy.maximumAudioSize }
      : { kind: "audio" };
  }

  if (isArchiveImage(descriptor, policy)) {
    return descriptor.sizeReal > policy.maximumImageSize
      ? { kind: "too-large", maximumSize: policy.maximumImageSize }
      : { kind: "image" };
  }

  if (!policy.extensions.some((candidate: string) => candidate === descriptor.extension)) {
    return { kind: "unsupported-extension", extension: descriptor.extension };
  }

  if (descriptor.sizeReal > policy.maximumSize) {
    return { kind: "too-large", maximumSize: policy.maximumSize };
  }

  return { kind: "supported" };
}
