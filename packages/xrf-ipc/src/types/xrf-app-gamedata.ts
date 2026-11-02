// Auto-generated rust bindings. Do not edit it manually.

import { XrayEngineChoice } from "#/types/xrf-engine-target";
import { GamedataVerificationType } from "#/types/xrf-gamedata";
import { JobOutcome } from "#/types/xrf-job";
import { Status } from "#/types/xrf-report";

/** Verifying a gamedata project. */
export const GAMEDATA_JOB_VERIFY = "gamedata.verify" as const;

/** One check's verdict, as the desktop surface shows it. */
export type GamedataCheckSummary = {
  /** The check that ran. */
  check: GamedataVerificationType;
  status: Status;
  /** The check's own one-line verdict. */
  summary: string;
  findings: number;
  /** How long this check took, where it measured itself. */
  duration: number | null;
};

/** What a verification was asked to do. */
export type GamedataVerifyRequest = {
  /** Gamedata root to verify. */
  root: string;
  /** Checks to run, or nothing for every one this build knows. */
  checks: Array<GamedataVerificationType> | null;
  /** Whether a check that would warn should fail instead. */
  isStrict: boolean;
  /** The engine the tree is meant for, which configs the engines read differently are read as; detected on `auto`. */
  engine: XrayEngineChoice;
};

/** What a whole verification reports back to the desktop surface. */
export type GamedataVerifySummary = {
  /** Whether every selected check finished, or the run stopped before or inside a check. */
  outcome: JobOutcome;
  /** The aggregate verdict over the checks that ran. */
  status: Status;
  checks: Array<GamedataCheckSummary>;
  duration: number;
};
