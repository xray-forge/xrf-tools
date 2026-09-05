import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";

import { describeUnpackOutcome } from "@/applications/archives-unpacker/lib/describe-unpack-outcome";
import { archivesCommands } from "@/core/bindings/commands/archives";
import { ArchiveUnpackResult } from "@/core/bindings/types/xrf-pack";
import { EJobKind, IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { Logger } from "@/lib/logging";
import { LatestFlow, TFlow } from "@/lib/mobx";

/**
 * The unpacking run and what it produced.
 *
 * A service rather than component state because an unpack outlives the view that started it: a user who navigates
 * away mid-run must be able to come back to it rather than find an idle form over work that is still writing files.
 */
@Injectable()
export class UnpackerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Cancellation keeps written files: they cannot be distinguished from the user's existing destination files. */
  public readonly operation: JobOperation<ArchiveUnpackResult>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [EJobKind.ARCHIVES_UNPACK], this.log);
  }

  /**
   * Unpacks every archive of a directory into a destination tree.
   *
   * @param source - Directory holding the packed archives.
   * @param destination - Directory the archives are unpacked into.
   */
  @LatestFlow()
  public *unpack(source: string, destination: string): TFlow {
    this.log.info("Unpacking:", source);

    yield* this.operation.run({
      kind: EJobKind.ARCHIVES_UNPACK,
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
