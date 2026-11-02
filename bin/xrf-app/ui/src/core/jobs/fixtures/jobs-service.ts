import { EventBus, WireEvent } from "@wirestate/core";
import { EJobsCommand } from "@xrf/ipc/commands/jobs";

import { IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs.service";
import { EMIT_NOTIFICATION_EVENT, INotificationPayload } from "@/core/notifications/lib";
import { IInjectedServiceMockDescriptor, mockInjectedService } from "@/fixtures/utils/container";

/** The commands the service lists the backend's jobs by and attaches to one by. */
export const JOBS_LIST_COMMAND: string = EJobsCommand.LIST;
export const JOBS_ATTACH_COMMAND: string = EJobsCommand.ATTACH;

/** How long the service waits between asking an adopted job where it has got to. */
export const JOBS_POLL_INTERVAL: number = 1000;

/** A listing slow enough that a test can provision again while it is still in the air. */
export const JOBS_SLOW_LISTING: number = 50;

/**
 * A provisioned service, plus whatever it announced.
 *
 * @returns The service and the notifications it raised.
 */
export function mockWatchedJobsService(): {
  service: JobsService;
  raised: Array<INotificationPayload>;
  settled: Array<IJobSettledPayload>;
} {
  const { service, container }: IInjectedServiceMockDescriptor<JobsService> = mockInjectedService(JobsService);
  const raised: Array<INotificationPayload> = [];
  const settled: Array<IJobSettledPayload> = [];

  container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
      raised.push(event.payload as INotificationPayload)
    );

  container
    .get(EventBus)
    .subscribe(JOB_SETTLED_EVENT, (event: WireEvent<IJobSettledPayload>) =>
      settled.push(event.payload as IJobSettledPayload)
    );

  return { service, raised, settled };
}
