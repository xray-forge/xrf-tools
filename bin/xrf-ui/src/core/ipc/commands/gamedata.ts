// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { GamedataVerifyRequest, GamedataVerifySummary } from "@/core/ipc/types/xrf-app";
import { JobProgress } from "@/core/ipc/types/xrf-job";

/** Commands */
export const gamedataCommands = {
  /** Run the selected checks over a gamedata project. */
  verifyProject: (request: GamedataVerifyRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<GamedataVerifySummary>("plugin:gamedata|verify_project", { request, jobId, progress }),
};
