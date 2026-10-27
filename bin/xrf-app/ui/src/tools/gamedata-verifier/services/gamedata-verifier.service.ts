import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { gamedataCommands } from "@xrf/ipc/commands/gamedata";
import { GAMEDATA_JOB_VERIFY, GamedataVerifySummary } from "@xrf/ipc/types/xrf-app-gamedata";
import { Logger } from "@xrf/logging";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { describeGamedataVerifyOutcome } from "@/tools/gamedata-verifier/lib/describe-gamedata-verify-outcome";
import { IGamedataVerification } from "@/tools/gamedata-verifier/lib/gamedata-verification";

/**
 * The gamedata verification run and what it found.
 */
@Injectable()
export class GamedataVerifierService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<GamedataVerifySummary> = new JobOperation(this.log, {
    kinds: [GAMEDATA_JOB_VERIFY],
  });

  /**
   * Runs every check this build knows over a gamedata root.
   *
   * @param verification - The root to verify, whether warnings fail it, and which engine it is meant for.
   */
  @ExclusiveFlow("operation")
  public *verify(verification: IGamedataVerification): TFlow {
    const { root, isStrict } = verification;

    this.log.info("Verifying gamedata:", root, isStrict);

    yield* this.operation.run({
      kind: GAMEDATA_JOB_VERIFY,
      // Every check this build knows: narrowing the selection is a refinement worth adding once somebody has watched a
      // full run and knows which one they want to repeat.
      invoke: (id: string, progress) => gamedataCommands.verifyProject({ ...verification, checks: null }, id, progress),
      describe: (outcome: IJobOutcome<GamedataVerifySummary>): IJobNotice =>
        describeGamedataVerifyOutcome(root, outcome),
    });
  }
}
