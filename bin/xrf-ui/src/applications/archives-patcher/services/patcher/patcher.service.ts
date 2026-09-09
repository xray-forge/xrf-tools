import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";

import { describePatchOutcome } from "@/applications/archives-patcher/lib/describe-patch-outcome";
import { archivesCommands } from "@/core/bindings/commands/archives";
import { ArchivesPatchRequest } from "@/core/bindings/types/xrf-app";
import { ArchivePatchConfig, ArchivePatchResult } from "@/core/bindings/types/xrf-pack";
import { EJobKind, IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { Logger } from "@/lib/logging";
import { ExclusiveFlow, TFlow } from "@/lib/mobx";

/**
 * Tracks comparison and patch jobs, including results adopted after navigation.
 */
@Injectable()
export class PatcherService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<ArchivePatchResult>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [EJobKind.ARCHIVES_COMPARE, EJobKind.ARCHIVES_PATCH], this.log);
  }

  /**
   * Compares two worlds and reports the difference, writing nothing.
   *
   * @param request - What to compare and where the difference would go.
   */
  @ExclusiveFlow("operation")
  public *compare(request: ArchivesPatchRequest): TFlow {
    yield* this.run(EJobKind.ARCHIVES_COMPARE, request);
  }

  /**
   * Compares two worlds and publishes the difference.
   *
   * @param request - What to compare and where to publish it.
   */
  @ExclusiveFlow("operation")
  public *patch(request: ArchivesPatchRequest): TFlow {
    yield* this.run(EJobKind.ARCHIVES_PATCH, request);
  }

  private *run(kind: EJobKind.ARCHIVES_COMPARE | EJobKind.ARCHIVES_PATCH, request: ArchivesPatchRequest): TFlow {
    if (this.operation.isRunning) {
      return;
    }

    const config: ArchivePatchConfig = request.config;

    this.log.info("Comparing:", config.base.length, "base root(s) against", config.target.length, "target root(s)");

    yield* this.operation.run({
      kind,
      invoke: (id: string, progress) =>
        kind === EJobKind.ARCHIVES_PATCH
          ? archivesCommands.patchArchives(request, id, progress)
          : archivesCommands.compareArchives(request, id, progress),
      describe: (outcome: IJobOutcome<ArchivePatchResult>): IJobNotice => describePatchOutcome(config, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
