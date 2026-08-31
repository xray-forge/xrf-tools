// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import { GamedataVerifyRequest, GamedataVerifySummary } from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";

/** Commands */
export const gamedataCommands = {
  /**
   * Run the selected checks over a gamedata project.
   *
   * Takes no lease: verification only reads, so two runs over one project have nothing to collide over. It is still a
   * job, because a full run over an installation is minutes of work that somebody may want to watch or call off.
   */
  verifyProject: (request: GamedataVerifyRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<GamedataVerifySummary>("plugin:gamedata|verify_project", { request, jobId, progress }),
};
