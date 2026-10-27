import { Injectable, OnDeprovision } from "@wirestate/core";
import { BoundAction, Observable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { KeybindCommand } from "@/core/commands";
import { OPEN_TOOL_HELP_KEYBIND_COMMAND } from "@/core/help/commands";

/**
 * Owns whether help is on screen, and which tool it belongs to.
 */
@Injectable()
export class HelpService {
  @Observable()
  public isOpen: boolean = false;

  /** Whether the open tool authored help at all; without it there is no affordance and no shortcut. */
  @Observable()
  public hasHelp: boolean = false;

  @Observable()
  private toolId: Nullable<string> = null;

  @OnDeprovision()
  public onDeprovision(): void {
    runInAction(() => {
      this.toolId = null;
      this.hasHelp = false;
      this.isOpen = false;
    });
  }

  /**
   * Publishes the tool help would describe.
   *
   * @param toolId - Tool currently routed, or null outside one.
   * @param hasHelp - Whether that tool authored help.
   */
  @BoundAction()
  public setTool(toolId: Nullable<string>, hasHelp: boolean): void {
    if (this.toolId !== toolId) {
      this.isOpen = false;
    }

    this.toolId = toolId;
    this.hasHelp = hasHelp;
  }

  @BoundAction()
  public close(): void {
    this.isOpen = false;
  }

  /** Shows help for whatever is open. */
  @KeybindCommand(OPEN_TOOL_HELP_KEYBIND_COMMAND, { isEnabled: (service: HelpService) => service.hasHelp })
  @BoundAction()
  public open(): void {
    this.isOpen = true;
  }
}
