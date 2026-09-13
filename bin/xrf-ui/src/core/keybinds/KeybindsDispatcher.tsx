import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect } from "react";

import { CommandsService, ICommandDescriptor } from "@/core/commands";
import { resolveKeybinding } from "@/core/keybinds/lib/keymap";
import { KeymapService } from "@/core/keybinds/services/keymap";
import { IApplicationDescriptor } from "@/core/routing/application";
import { useCurrentApplication } from "@/core/routing/current-application.context";
import { SettingsService } from "@/core/settings/services/settings";
import { isTextEntryTarget } from "@/lib/dom/text-entry";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Turns key events into commands, for the whole window.
 */
export function KeybindsDispatcher(): Nullable<ReactElement> {
  const commandsService: CommandsService = useInjection(CommandsService);
  const keymapService: KeymapService = useInjection(KeymapService);
  const settingsService: SettingsService = useInjection(SettingsService);
  const application: Nullable<IApplicationDescriptor> = useCurrentApplication();

  useEffect(() => keymapService.setApplication(application), [application, keymapService]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent): void {
      // A held key repeats; an action is not something to run sixty times because a finger stayed down.
      if (event.defaultPrevented || event.repeat) {
        return;
      }

      // Read at the event rather than captured: the services are stable, so the listener attaches once and still
      // sees the current application and bindings.
      const command: Nullable<ICommandDescriptor> = resolveKeybinding(
        keymapService.keymap,
        event,
        isTextEntryTarget(event.target)
      );

      if (!command) {
        return;
      }

      // Only a command that actually ran consumes the key: a chord swallowed by a guard that refused it, or by a
      // command nothing implements, would be a shortcut that silently disables a key the webview still needs.
      if (commandsService.execute(command)) {
        event.preventDefault();
      } else if (settingsService.isDevModeEnabled) {
        Logger.warn("Keybind resolved to a command nothing can run:", command.id, command.chords);
      }
    }

    window.addEventListener("keydown", onKeyDown);

    return () => window.removeEventListener("keydown", onKeyDown);
  }, [commandsService, keymapService, settingsService]);

  return null;
}
