import {
  ArchiveDescribeScope,
  ArchiveReference,
  EArchiveDescribeScope,
  EArchiveReferenceStatus,
} from "@/core/ipc/types/xrf-app";
import { assertExhaustive } from "@/lib/types/exhaustive";
import { Nullable } from "@/lib/types/general";

/** Shown where a file declares no value at all, which is not the same as declaring a default one. */
export const NOT_DECLARED: string = "Not declared";

/**
 * What became of a reference, worded in the terms of the subject that was searched.
 *
 * @param reference - Resolved reference to describe.
 * @param scope - What the lookup behind it searched.
 * @returns A phrase for the reference's caption, or null when the reference resolved and needs no qualifying.
 */
export function describeReferenceStatus(reference: ArchiveReference, scope: ArchiveDescribeScope): Nullable<string> {
  switch (reference.status) {
    case EArchiveReferenceStatus.PRESENT:
      return reference.path ?? null;
    case EArchiveReferenceStatus.ABSENT:
      return scope.kind === EArchiveDescribeScope.WORLD
        ? "Not found in the mounted tree"
        : `Not in ${scope.volumes === 1 ? "this volume" : `these ${scope.volumes} volumes`}`;
    case EArchiveReferenceStatus.UNKNOWN:
      return "Not a name that can be looked up";
    default:
      return assertExhaustive(reference.status);
  }
}

/**
 * A texture param colour as the word it is stored as.
 *
 * @param value - Colour word from the descriptor.
 * @returns The word in the spelling an author would compare against.
 */
export function formatColorWord(value: number): string {
  return `0x${(value >>> 0).toString(16).padStart(8, "0").toUpperCase()}`;
}

/**
 * A chunk id in the spelling `ETextureParams.h` gives it.
 *
 * @param id - Chunk id as read.
 * @returns The id as four hex digits.
 */
export function formatChunkId(id: number): string {
  return `0x${(id >>> 0).toString(16).padStart(4, "0").toUpperCase()}`;
}
