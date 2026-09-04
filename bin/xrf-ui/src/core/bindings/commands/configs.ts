// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import { ConfigsFormatRequest, ConfigsVerifyRequest } from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import { LtxProjectFormatResult, LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";

/** Commands */
export const configsCommands = {
  /**
   * Report which LTX configs roots exposes are misformatted.
   *
   * Read-only, so an archived config is checked like any other, and no lease is taken: two readers of one project have
   * nothing to collide over. A separate kind from the rewrite it reports on, because they are different work with
   * different consequences — one answers a question, the other changes the files.
   */
  checkDirectoryFormat: (request: ConfigsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectFormatResult>("plugin:configs|check_directory_format", { request, jobId, progress }),
  /**
   * Rewrite the LTX configs roots exposes.
   *
   * Writing needs a file, so this refuses a project holding archived winners — the refusal comes from
   * `xrf-ltx` itself. Formatting an installation is therefore a legitimate refusal, not a gap.
   *
   * Holds the roots exclusively for the whole run, so a second request over the same set is refused rather than allowed
   * to rewrite the files this one is walking. A cancelled run leaves the files it had already formatted formatted and
   * the rest untouched: each file is rewritten through a staged replace, so nothing is half-written and running it
   * again resolves the difference.
   */
  formatDirectory: (request: ConfigsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectFormatResult>("plugin:configs|format_directory", { request, jobId, progress }),
  /**
   * Verify the LTX configs roots exposes.
   *
   * Read-only, so it goes through the roots and covers archived configs too. `xrf-ltx` draws the same
   * line: its read-only check reads through the VFS where its rewrite refuses archived winners.
   */
  verifyDirectory: (request: ConfigsVerifyRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectVerifyResult>("plugin:configs|verify_directory", { request, jobId, progress }),
};
