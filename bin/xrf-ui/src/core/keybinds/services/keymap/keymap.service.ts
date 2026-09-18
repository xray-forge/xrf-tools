import { Injectable, OnDeprovision } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { IKeybindCommand } from "@/core/commands";
import { ROOT_KEYBIND_COMMANDS } from "@/core/commands/root-commands";
import { buildKeymap, IKeymap } from "@/core/keybinds/lib/keymap";
import { LAUNCHER_KEYBIND_COMMANDS } from "@/core/launcher/commands";
import { IApplicationDescriptor } from "@/core/routing/application";
import { KEYBINDS_STORAGE_KEY } from "@/core/storage";
import { parseLocalStorageValueSafe, setLocalStorageValueSafe } from "@/lib/local-storage";
import { Logger } from "@/lib/logging";
import { EMPTY_ARRAY } from "@/lib/types/array";
import { Nullable } from "@/lib/types/general";

/** Stored shape of the overlay: command id to the chords bound in its place. */
type TKeybindOverrides = Readonly<Record<string, ReadonlyArray<string>>>;

/**
 * Owns what the keyboard currently does: which commands are reachable, and which chords each answers to.
 */
@Injectable()
export class KeymapService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  private overrides: TKeybindOverrides = KeymapService.readOverrides();

  @Observable()
  private application: Nullable<IApplicationDescriptor> = null;

  /**
   * Reads the overlay, discarding entries it cannot use rather than the whole map.
   *
   * @returns Usable overrides, or an empty map.
   */
  private static readOverrides(): TKeybindOverrides {
    const stored: unknown = parseLocalStorageValueSafe(KEYBINDS_STORAGE_KEY);

    if (stored === null || typeof stored !== "object" || Array.isArray(stored)) {
      return {};
    }

    const usable: Record<string, ReadonlyArray<string>> = {};

    for (const [id, chords] of Object.entries(stored as Record<string, unknown>)) {
      if (Array.isArray(chords) && chords.every((chord: unknown) => typeof chord === "string")) {
        usable[id] = chords as Array<string>;
      } else {
        Logger.warn("Discarding unusable keybind override:", id);
      }
    }

    return usable;
  }

  /**
   * Every command reachable right now: the root ones, plus whatever the open screen declares.
   */
  @Computed()
  public get commands(): ReadonlyArray<IKeybindCommand> {
    const byId: Map<string, IKeybindCommand> = new Map();
    const screen: ReadonlyArray<IKeybindCommand> = this.application
      ? (this.application.keybindCommands ?? EMPTY_ARRAY)
      : LAUNCHER_KEYBIND_COMMANDS;

    for (const command of [...ROOT_KEYBIND_COMMANDS, ...screen]) {
      byId.set(command.id, command);
    }

    return [...byId.values()];
  }

  /** The lookup the dispatcher scans, rebuilt only when the reachable set or a binding changes. */
  @Computed()
  public get keymap(): IKeymap {
    return buildKeymap(this.commands, (command: IKeybindCommand) => this.getChords(command));
  }

  @OnDeprovision()
  public onDeprovision(): void {
    this.setApplication(null);
  }

  /**
   * Publishes the routed application, whose declarations decide what is reachable beside the root commands.
   *
   * @param application - Application currently routed, or null outside one.
   */
  @BoundAction()
  public setApplication(application: Nullable<IApplicationDescriptor>): void {
    this.application = application;
  }

  /**
   * The chords a command currently answers to.
   *
   * @param command - Command to resolve.
   * @returns Its effective chords.
   */
  public getChords(command: IKeybindCommand): ReadonlyArray<string> {
    return this.overrides[command.id] ?? command.chords;
  }

  /**
   * The chords a command answers where it is reachable, and none where it is not.
   *
   * @param command - Command to resolve.
   * @returns Its effective chords, or nothing when this screen does not declare it.
   */
  public getReachableChords(command: IKeybindCommand): ReadonlyArray<string> {
    return this.commands.some((it: IKeybindCommand) => it.id === command.id) ? this.getChords(command) : [];
  }

  /**
   * Binds a command to chords of someone's choosing, or restores its declared ones.
   *
   * @param commandId - Command to rebind.
   * @param chords - Chords to bind, or null to restore the declared defaults.
   */
  @BoundAction()
  public setChords(commandId: string, chords: Nullable<ReadonlyArray<string>>): void {
    const next: Record<string, ReadonlyArray<string>> = { ...this.overrides };

    if (chords === null) {
      delete next[commandId];
    } else {
      next[commandId] = [...chords];
    }

    this.overrides = next;
    this.log.info("Rebound command:", commandId, chords);

    // An emptied overlay removes the key rather than storing `{}`, so nothing declared-only leaves a trace.
    setLocalStorageValueSafe(KEYBINDS_STORAGE_KEY, Object.keys(next).length > 0 ? JSON.stringify(next) : null);
  }
}
