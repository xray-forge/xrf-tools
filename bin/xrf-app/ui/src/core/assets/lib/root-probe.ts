import { EXrayRootKind, XrayRootProbe } from "@xrf/ipc/types/xrf-vfs";
import { assertExhaustive } from "@xrf/types";

/**
 * What a probed path is, in one line meant to be read beside the path itself.
 *
 * @param probe - What the backend made of the path.
 * @returns A description of the path.
 */
export function describeRootProbe(probe: XrayRootProbe): string {
  switch (probe.kind) {
    case EXrayRootKind.INSTALLATION:
      return `Game installation, ${countOf(probe.mounts, "source")}`;

    case EXrayRootKind.VOLUMES:
      return "Archive volumes";

    case EXrayRootKind.ROOT:
      return `Game data: ${probe.evidence.join(", ")}`;

    case EXrayRootKind.UNRECOGNIZED:
      return "Nothing here looks like game data";

    case EXrayRootKind.MISSING:
      return "Path does not exist";

    default:
      return assertExhaustive(probe.kind);
  }
}

function countOf(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? "" : "s"}`;
}
