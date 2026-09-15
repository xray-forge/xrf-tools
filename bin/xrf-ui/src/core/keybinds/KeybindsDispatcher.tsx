import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect } from "react";

import { IKeybindCommand, KeybindCommandsService } from "@/core/commands";
import { resolveKeybinding } from "@/core/keybinds/lib/keymap";
import { KeymapService } from "@/core/keybinds/services/keymap";
import { IApplicationDescriptor } from "@/core/routing/application";
import { useCurrentApplication } from "@/core/routing/current-application.context";
import { SettingsService } from "@/core/settings/services/settings";
import { isModalOpen } from "@/lib/dom/modal";
import { isTextEntryTarget } from "@/lib/dom/text-entry";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Turns key events into commands, for the whole window.
 */
export function KeybindsDispatcher(): Nullable<ReactElement> {
  const commandsService: KeybindCommandsService = useInjection(KeybindCommandsService);
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
      const command: Nullable<IKeybindCommand> = resolveKeybinding(
        keymapService.keymap,
        event,
        isTextEntryTarget(event.target)
      );

      if (!command) {
        return;
      }

      // A modal traps focus, so the application behind it cannot be reached by pointer; a chord must not reach it
      // either. Probed only once a chord matched, which is rare, rather than on every key someone types.
      if (isModalOpen()) {
        return;
      }

      // Only a command that actually ran consumes the key: a chord swallowed by a guard that refused it, or by a
      // command nothing implements, would be a shortcut that silently disables a key the webview still needs.
      if (commandsService.execute(command)) {
        event.preventDefault();
      } else if (settingsService.isDevModeEnabled && !commandsService.isImplemented(command)) {
        // Only the unimplemented case is worth saying out loud. A guard refusing is enablement doing its job, and
        // warning about it would make every correctly guarded command noise.
        Logger.warn("Keybind resolved to a command nothing implements:", command.id, command.chords);
      }
    }

    window.addEventListener("keydown", onKeyDown);

    return () => window.removeEventListener("keydown", onKeyDown);
  }, [commandsService, keymapService, settingsService]);

  return null;
}
