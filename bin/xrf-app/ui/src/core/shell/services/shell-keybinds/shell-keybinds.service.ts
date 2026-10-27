import { inject, Injectable } from "@wirestate/core";

import { KeybindCommand } from "@/core/commands";
import { CLOSE_TOOL_KEYBIND_COMMAND } from "@/core/shell/commands";
import { ShellToolService } from "@/core/shell/services/shell-tool";
import { ToolLifecycleService } from "@/core/shell/services/tool-lifecycle";

/**
 * Answers the shell's commands, which every screen reaches.
 */
@Injectable()
export class ShellKeybindsService {
  public constructor(
    private readonly lifecycleService: ToolLifecycleService = inject(ToolLifecycleService),
    private readonly shellToolService: ShellToolService = inject(ShellToolService)
  ) {}

  /** Whether a tool is open and nothing it runs holds it there. */
  public get canCloseTool(): boolean {
    return !this.lifecycleService.isBusy && this.shellToolService.isToolOpen;
  }

  /**
   * Goes back to all tools, once unsaved work is saved or let go.
   */
  @KeybindCommand(CLOSE_TOOL_KEYBIND_COMMAND, { isEnabled: (service: ShellKeybindsService) => service.canCloseTool })
  public closeTool(): void {
    this.lifecycleService.requestLeave(this.shellToolService.close);
  }
}
