import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { archivesCommands } from "@xrf/ipc/commands/archives";
import { ARCHIVES_JOB_UNPACK, ArchivesUnpackRequest } from "@xrf/ipc/types/xrf-app-archives";
import { ArchiveUnpackResult } from "@xrf/ipc/types/xrf-pack";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { TCommandCompletion } from "@/core/operations/lib/command-reporter";
import { describeUnpackOutcome } from "@/tools/archives-unpacker/lib/describe-unpack-outcome";

/**
 * The unpacking run and what it produced.
 */
@Injectable()
export class ArchivesUnpackerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Cancellation keeps written files: they cannot be distinguished from the user's existing destination files. */
  public readonly operation: JobOperation<ArchiveUnpackResult> = new JobOperation(this.log, {
    kinds: [ARCHIVES_JOB_UNPACK],
  });

  /**
   * Unpacks every archive of a directory into a destination tree.
   *
   * @param request - The directory holding the packed archives, the one they are unpacked into, and whether files
   *   already there are replaced; the backend refuses otherwise.
   * @returns How the run ended, or null where none started.
   */
  @ExclusiveFlow("operation")
  public *unpack(request: ArchivesUnpackRequest): TFlow<Nullable<TCommandCompletion<ArchiveUnpackResult>>> {
    const { from: source, destination } = request;

    this.log.info("Unpacking:", source, "to", destination);

    return yield* this.operation.run({
      kind: ARCHIVES_JOB_UNPACK,
      invoke: (id: string, progress) => archivesCommands.unpackDirectory(request, id, progress),
      describe: (outcome: IJobOutcome<ArchiveUnpackResult>): IJobNotice =>
        describeUnpackOutcome(source, destination, outcome),
    });
  }
}
