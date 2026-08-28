import { XrayRootProbe } from "@/core/bindings/types/xrf-vfs";

/**
 * What a probed path is, in one line meant to be read beside the path itself.
 *
 * @param probe - What the backend made of the path.
 * @returns A description of the path.
 */
export function describeRootProbe(probe: XrayRootProbe): string {
  switch (probe.kind) {
    case "installation":
      return `Game installation, ${countOf(probe.mounts, "source")}`;

    case "volumes":
      return "Archive volumes";

    case "root":
      return `Game data: ${probe.evidence.join(", ")}`;

    case "unrecognized":
      return "Nothing here looks like game data";

    case "missing":
      return "Path does not exist";
  }
}

function countOf(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? "" : "s"}`;
}
