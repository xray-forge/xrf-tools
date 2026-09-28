import { Maybe } from "@xrf/types";

import { EBuildKind } from "@/core/ipc/types/xrf-build-info";

/**
 * Kind of build the running binary is, as it told the document before any script ran.
 *
 * A document no binary hosts, under plain vite or a test runner, reads as local: that is where one runs.
 */
export function getBuildKind(): EBuildKind {
  const recorded: Maybe<string> = typeof window === "undefined" ? undefined : window.__XRF_BUILD_KIND__;

  return Object.values(EBuildKind).find((kind) => kind === recorded) ?? EBuildKind.LOCAL;
}
