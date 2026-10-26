import { CommandBus, inject, Injectable, QueryBus } from "@wirestate/core";

import { KeybindCommand } from "@/core/commands";
import { CLOSE_TOOL_KEYBIND_COMMAND } from "@/core/shell/commands";
import { EditorLifecycleService } from "@/core/shell/services/editor-lifecycle";
import { APPLICATION_OPEN_QUERY, CLOSE_APPLICATION_COMMAND } from "@/core/shell/shell-messages";

/**
 * Answers the shell's commands, which every screen reaches.
 */
@Injectable()
export class ShellKeybindsService {
  public constructor(
    private readonly commandBus: CommandBus = inject(CommandBus),
    private readonly queryBus: QueryBus = inject(QueryBus),
    private readonly lifecycleService: EditorLifecycleService = inject(EditorLifecycleService)
  ) {}

  /** Whether a tool is open and nothing it runs holds it there. */
  public get canCloseTool(): boolean {
    return (
      !this.lifecycleService.isBusy &&
      (this.queryBus.query<boolean>(APPLICATION_OPEN_QUERY, undefined, { optional: true }) ?? false)
    );
  }

  /**
   * Goes back to all tools, once unsaved work is saved or let go.
   */
  @KeybindCommand(CLOSE_TOOL_KEYBIND_COMMAND, { isEnabled: (service: ShellKeybindsService) => service.canCloseTool })
  public closeTool(): void {
    this.lifecycleService.requestLeave(() =>
      this.commandBus.execute(CLOSE_APPLICATION_COMMAND, undefined, { optional: true })
    );
  }
}
