import { inject, Injectable, OnDeprovision, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, Observable, reaction, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { KeybindCommand } from "@/core/commands";
import { OPEN_TOOL_HELP_KEYBIND_COMMAND } from "@/core/help/commands";
import { ILauncherScreen, LAUNCHER_SCREEN } from "@/core/routing/launcher-screen";
import { ShellToolService } from "@/core/routing/services/shell-tool.service";
import { IToolHelp } from "@/core/routing/tool";

/**
 * Owns the help for whatever is on screen, and whether it is open; it closes as the screen it was opened on goes.
 */
@Injectable()
export class HelpService {
  @Observable()
  public isOpen: boolean = false;

  private stopFollowing: Nullable<() => void> = null;

  public constructor(
    private readonly shellToolService: ShellToolService = inject(ShellToolService),
    private readonly launcher: ILauncherScreen = inject(LAUNCHER_SCREEN)
  ) {}

  /** The help the open screen authored: the tool's, the launcher's, or none on a route nothing answers. */
  @Computed()
  public get help(): Nullable<IToolHelp> {
    const { tool, isLauncher } = this.shellToolService;

    if (tool) {
      return tool.help ?? null;
    }

    return isLauncher ? this.launcher.help : null;
  }

  /** Whether the open screen authored help at all; without it there is no affordance and no shortcut. */
  @Computed()
  public get hasHelp(): boolean {
    return this.help !== null;
  }

  @OnProvision()
  public onProvision(): void {
    this.stopFollowing?.();
    // Another screen's help is other help; one opened over a screen never reopens over the next.
    this.stopFollowing = reaction(
      () => this.help,
      () => this.close()
    );
  }

  @OnDeprovision()
  public onDeprovision(): void {
    this.stopFollowing?.();
    this.stopFollowing = null;

    runInAction(() => {
      this.isOpen = false;
    });
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
