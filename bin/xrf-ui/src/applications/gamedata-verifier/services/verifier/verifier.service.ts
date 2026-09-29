import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";

import { describeGamedataVerifyOutcome } from "@/applications/gamedata-verifier/lib/describe-gamedata-verify-outcome";
import { IGamedataVerification } from "@/applications/gamedata-verifier/lib/gamedata-verification";
import { gamedataCommands } from "@/core/ipc/commands/gamedata";
import { EJobKind, GamedataVerifySummary } from "@/core/ipc/types/xrf-app";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { Logger } from "@/lib/logging";
import { ExclusiveFlow, TFlow } from "@/lib/mobx";

/**
 * The gamedata verification run and what it found.
 */
@Injectable()
export class GamedataVerifierService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<GamedataVerifySummary>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [EJobKind.GAMEDATA_VERIFY]);
  }

  /**
   * Runs every check this build knows over a gamedata root.
   *
   * @param verification - The root to verify, whether warnings fail it, and which engine it is meant for.
   */
  @ExclusiveFlow("operation")
  public *verify(verification: IGamedataVerification): TFlow {
    const { root, isStrict } = verification;

    if (this.operation.isRunning) {
      return;
    }

    this.log.info("Verifying gamedata:", root, isStrict);

    yield* this.operation.run({
      kind: EJobKind.GAMEDATA_VERIFY,
      // Every check this build knows: narrowing the selection is a refinement worth adding once somebody has watched a
      // full run and knows which one they want to repeat.
      invoke: (id: string, progress) => gamedataCommands.verifyProject({ ...verification, checks: null }, id, progress),
      describe: (outcome: IJobOutcome<GamedataVerifySummary>): IJobNotice =>
        describeGamedataVerifyOutcome(root, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
