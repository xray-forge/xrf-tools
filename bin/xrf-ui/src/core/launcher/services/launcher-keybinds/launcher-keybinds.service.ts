import { CommandBus, inject, Injectable } from "@wirestate/core";

import { KeybindCommand } from "@/core/commands";
import { FOCUS_SEARCH_KEYBIND_COMMAND } from "@/core/search/commands";
import { FOCUS_SEARCH_FIELD_MESSAGE } from "@/core/search/lib";

/**
 * Answers the home screen's commands, for as long as the home screen is on screen.
 */
@Injectable()
export class LauncherKeybindsService {
  public constructor(private readonly commandBus: CommandBus = inject(CommandBus)) {}

  /**
   * Hands the caret to the catalog's field.
   */
  @KeybindCommand(FOCUS_SEARCH_KEYBIND_COMMAND, {
    isEnabled: (service: LauncherKeybindsService) => service.commandBus.hasHandler(FOCUS_SEARCH_FIELD_MESSAGE),
  })
  public focusSearch(): void {
    this.commandBus.execute(FOCUS_SEARCH_FIELD_MESSAGE, undefined, { optional: true });
  }
}
