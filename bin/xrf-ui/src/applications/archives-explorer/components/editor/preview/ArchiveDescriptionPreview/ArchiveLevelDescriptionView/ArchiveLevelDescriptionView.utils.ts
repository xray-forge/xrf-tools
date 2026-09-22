import { assertExhaustive, Nullable } from "@xrf/types";

import {
  ArchiveDescribeScope,
  ArchiveLevelShader,
  EArchiveDescribeScope,
  EArchiveReferenceStatus,
} from "@/core/ipc/types/xrf-app";

/**
 * What became of a shader name, worded for a definition rather than for a file.
 *
 * @param shader - Shader name as the surface carries it.
 * @param scope - What the lookups behind the description searched.
 * @returns A phrase qualifying the name, or null when it resolved and needs none.
 */
export function describeShaderStatus(shader: ArchiveLevelShader, scope: ArchiveDescribeScope): Nullable<string> {
  switch (shader.status) {
    case EArchiveReferenceStatus.PRESENT:
      return null;
    case EArchiveReferenceStatus.ABSENT:
      return "Not defined by the shader library";
    case EArchiveReferenceStatus.UNKNOWN:
      return scope.kind === EArchiveDescribeScope.WORLD
        ? "No shader library in the mounted tree to ask"
        : "No shader library in these volumes to ask";
    default:
      return assertExhaustive(shader.status);
  }
}
