import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { archivesCommands } from "@xrf/ipc/commands/archives";
import { ARCHIVES_JOB_UNPACK } from "@xrf/ipc/types/xrf-app-archives";
import { ArchiveUnpackResult } from "@xrf/ipc/types/xrf-pack";
import { Logger } from "@xrf/logging";

import { describeUnpackOutcome } from "@/applications/archives-unpacker/lib/describe-unpack-outcome";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";

/**
 * The unpacking run and what it produced.
 */
@Injectable()
export class UnpackerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Cancellation keeps written files: they cannot be distinguished from the user's existing destination files. */
  public readonly operation: JobOperation<ArchiveUnpackResult>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [ARCHIVES_JOB_UNPACK]);
  }

  /**
   * Unpacks every archive of a directory into a destination tree.
   *
   * @param source - Directory holding the packed archives.
   * @param destination - Directory the archives are unpacked into.
   */
  @ExclusiveFlow("operation")
  public *unpack(source: string, destination: string): TFlow {
    if (this.operation.isRunning) {
      return;
    }

    this.log.info("Unpacking:", source, "to", destination);

    yield* this.operation.run({
      kind: ARCHIVES_JOB_UNPACK,
      invoke: (id: string, progress) => archivesCommands.unpackDirectory({ from: source, destination }, id, progress),
      describe: (outcome: IJobOutcome<ArchiveUnpackResult>): IJobNotice =>
        describeUnpackOutcome(source, destination, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
