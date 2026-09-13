import { CommandBus, EventBus, inject, Injectable, QueryBus } from "@wirestate/core";

import { ICommandDescriptor, toCommandEnabledQuery } from "@/core/commands/lib/command-descriptor";
import { transformError } from "@/core/error/lib";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { APPLICATION_SOURCE } from "@/core/routing/application";
import { Logger } from "@/lib/logging";
import { isCancellation } from "@/lib/mobx";

/** Invokes declared commands and answers whether one can be invoked, for every surface that offers them. */
@Injectable()
export class CommandsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public constructor(
    private readonly commandBus: CommandBus = inject(CommandBus),
    private readonly queryBus: QueryBus = inject(QueryBus),
    private readonly eventBus: EventBus = inject(EventBus)
  ) {}

  /**
   * Whether something is bound to handle this command and that handler's own guard allows it now.
   *
   * @param descriptor - Command to test.
   * @returns Whether invoking it would do anything.
   */
  public isAvailable(descriptor: ICommandDescriptor): boolean {
    if (!this.commandBus.hasHandler(descriptor.id)) {
      return false;
    }

    return this.queryBus.query<boolean>(toCommandEnabledQuery(descriptor), undefined, { optional: true }) ?? true;
  }

  /**
   * Runs a command when it is available, reporting a failure rather than raising it.
   *
   * @param descriptor - Command to run.
   * @returns Whether the command was dispatched.
   */
  public execute(descriptor: ICommandDescriptor): boolean {
    if (!this.isAvailable(descriptor)) {
      return false;
    }

    this.log.info("Executing command:", descriptor.id);

    Promise.resolve(this.commandBus.executeAsync(descriptor.id, undefined, { optional: true })).catch(
      (error: unknown) => {
        // A superseded flow rejects on purpose; reporting it would name the command that replaced this one a failure.
        if (isCancellation(error)) {
          return;
        }

        this.log.error("Command failed:", descriptor.id, error);

        emitNotification(this.eventBus, {
          details: transformError(error).message,
          severity: ENotificationSeverity.ERROR,
          source: APPLICATION_SOURCE,
          title: `Could not ${descriptor.label.toLowerCase()}`,
        });
      }
    );

    return true;
  }
}
