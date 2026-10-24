// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { GamedataVerifyRequest, GamedataVerifySummary } from "#/types/xrf-app-gamedata";
import { JobProgress } from "#/types/xrf-job";

/** The wire name of every command of the `gamedata` plugin. */
export enum EGamedataCommand {
  VERIFY_PROJECT = "plugin:gamedata|verify_project",
}

/** Commands */
export const gamedataCommands = {
  /** Run the selected checks over a gamedata project, one verification at a time. */
  verifyProject: (request: GamedataVerifyRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<GamedataVerifySummary>(EGamedataCommand.VERIFY_PROJECT, { request, jobId, progress }),
};

/** What each command of the `gamedata` plugin takes and resolves to, keyed by its wire name. */
export interface IGamedataCommandSignatures {
  [EGamedataCommand.VERIFY_PROJECT]: IIpcCommandSignature<
    typeof gamedataCommands.verifyProject,
    ["request", "jobId", "progress"]
  >;
}
