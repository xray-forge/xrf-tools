import { formatCount } from "@xrf/format";
import { ArchivePatchResult } from "@xrf/ipc/types/xrf-pack";

/**
 * Summarizes the comparison or publication result.
 *
 * @param result - Completed run report.
 * @returns A headline for the publication outcome.
 */
export function describePatchHeadline(result: ArchivePatchResult): string {
  const carried: number = result.added.length + result.modified.length;

  switch (result.publication.kind) {
    case "published":
      return `Published ${formatCount(carried)} entry(s) into ${result.publication.volumes.length} volume(s)`;

    case "unnecessary":
      return "Nothing differs, so no patch was written";

    default:
      return carried
        ? `${formatCount(carried)} entry(s) would be carried into a patch`
        : "Nothing differs between the two worlds";
  }
}
