import { Injectable, OnDeprovision } from "@wirestate/core";
import { BoundAction, Observable, runInAction } from "@wirestate/mobx";

import { Command } from "@/core/commands";
import { OPEN_APPLICATION_HELP_COMMAND } from "@/core/help/commands";
import { Nullable } from "@/lib/types/general";

/**
 * Owns whether help is on screen, and which application it belongs to.
 */
@Injectable()
export class HelpService {
  @Observable()
  public isOpen: boolean = false;

  /** Whether the open application authored help at all; without it there is no affordance and no shortcut. */
  @Observable()
  public hasHelp: boolean = false;

  @Observable()
  private applicationId: Nullable<string> = null;

  @OnDeprovision()
  public onDeprovision(): void {
    runInAction(() => {
      this.applicationId = null;
      this.hasHelp = false;
      this.isOpen = false;
    });
  }

  /**
   * Publishes the application help would describe.
   *
   * @param applicationId - Application currently routed, or null outside one.
   * @param hasHelp - Whether that application authored help.
   */
  @BoundAction()
  public setApplication(applicationId: Nullable<string>, hasHelp: boolean): void {
    if (this.applicationId !== applicationId) {
      this.isOpen = false;
    }

    this.applicationId = applicationId;
    this.hasHelp = hasHelp;
  }

  @BoundAction()
  public close(): void {
    this.isOpen = false;
  }

  /** Shows help for whatever is open. */
  @Command(OPEN_APPLICATION_HELP_COMMAND, { isEnabled: (service: HelpService) => service.hasHelp })
  @BoundAction()
  public open(): void {
    this.isOpen = true;
  }
}
