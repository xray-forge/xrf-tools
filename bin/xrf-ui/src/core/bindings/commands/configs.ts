// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import { LtxProjectFormatResult, LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const configsCommands = {
  /**
   * Report which LTX configs roots exposes are misformatted.
   *
   * Read-only, so an archived config is checked like any other.
   */
  checkDirectoryFormat: (roots: XrayRoots, prefix: string | null) =>
    __TAURI_INVOKE<LtxProjectFormatResult>("plugin:configs|check_directory_format", { roots, prefix }),
  /**
   * Rewrite the LTX configs roots exposes.
   *
   * Writing needs a file, so this refuses a project holding archived winners — the refusal comes from
   * `xrf-ltx` itself. Formatting an installation is therefore a legitimate refusal, not a gap.
   */
  formatDirectory: (roots: XrayRoots, prefix: string | null) =>
    __TAURI_INVOKE<LtxProjectFormatResult>("plugin:configs|format_directory", { roots, prefix }),
  /**
   * Verify the LTX configs roots exposes.
   *
   * Read-only, so it goes through the roots and covers archived configs too. `xrf-ltx` draws the same
   * line: its read-only check reads through the VFS where its rewrite refuses archived winners.
   */
  verifyDirectory: (roots: XrayRoots, prefix: string | null) =>
    __TAURI_INVOKE<LtxProjectVerifyResult>("plugin:configs|verify_directory", { roots, prefix }),
};
