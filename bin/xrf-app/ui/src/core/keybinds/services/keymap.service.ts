import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed } from "@wirestate/mobx";
import { EMPTY_ARRAY } from "@xrf/collections";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { IKeybindCommand } from "@/core/commands";
import { ROOT_KEYBIND_COMMANDS } from "@/core/commands/root-commands";
import { parseChord } from "@/core/keybinds/lib/chord";
import { buildKeymap, IKeymap } from "@/core/keybinds/lib/keymap";
import { LAUNCHER_KEYBIND_COMMANDS } from "@/core/launcher/commands";
import { IToolDescriptor } from "@/core/routing/tool";
import { ShellToolService } from "@/core/shell/services/shell-tool";
import { jsonCodec, KEYBINDS_STORAGE_KEY, StoredValue } from "@/core/storage";

/** Stored shape of the overlay: command id to the chords bound in its place. */
type TKeybindOverrides = Readonly<Record<string, ReadonlyArray<string>>>;

/**
 * Owns what the keyboard currently does: which commands are reachable, and which chords each answers to.
 */
@Injectable()
export class KeymapService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly overrides: StoredValue<TKeybindOverrides> = new StoredValue(
    KEYBINDS_STORAGE_KEY,
    jsonCodec(KeymapService.readOverrides, KeymapService.writeOverrides)
  );

  public constructor(private readonly shellToolService: ShellToolService = inject(ShellToolService)) {}

  /**
   * Reads the overlay, discarding entries it cannot use rather than the whole map.
   *
   * @param stored - The parsed overlay, `null` when none was stored.
   * @returns Usable overrides, or an empty map.
   */
  private static readOverrides(stored: unknown): TKeybindOverrides {
    if (stored === null || typeof stored !== "object" || Array.isArray(stored)) {
      return {};
    }

    const usable: Record<string, ReadonlyArray<string>> = {};

    for (const [id, chords] of Object.entries(stored as Record<string, unknown>)) {
      if (!Array.isArray(chords) || !chords.every((chord: unknown) => typeof chord === "string")) {
        Logger.warn("Discarding unusable keybind override:", id);

        continue;
      }

      try {
        for (const chord of chords) {
          parseChord(chord);
        }
      } catch (error: unknown) {
        Logger.warn("Discarding unusable keybind override:", id, error);

        continue;
      }

      usable[id] = chords;
    }

    return usable;
  }

  /**
   * @param overrides - The overlay to keep.
   * @returns The overlay as JSON; an emptied one removes the key rather than storing `{}`, so nothing declared-only
   *   leaves a trace.
   */
  private static writeOverrides(overrides: TKeybindOverrides): Nullable<string> {
    return Object.keys(overrides).length > 0 ? JSON.stringify(overrides) : null;
  }

  /**
   * Every command reachable right now: the root ones, plus whatever the open screen declares.
   */
  @Computed()
  public get commands(): ReadonlyArray<IKeybindCommand> {
    const byId: Map<string, IKeybindCommand> = new Map();
    const tool: Nullable<IToolDescriptor> = this.shellToolService.tool;
    const screen: ReadonlyArray<IKeybindCommand> = tool
      ? (tool.keybindCommands ?? EMPTY_ARRAY)
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

  /**
   * The chords a command currently answers to.
   *
   * @param command - Command to resolve.
   * @returns Its effective chords.
   */
  public getChords(command: IKeybindCommand): ReadonlyArray<string> {
    return this.overrides.value[command.id] ?? command.chords;
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
    const next: Record<string, ReadonlyArray<string>> = { ...this.overrides.value };

    if (chords === null) {
      delete next[commandId];
    } else {
      next[commandId] = [...chords];
    }

    this.overrides.set(next);
    this.log.info("Rebound command:", commandId, chords);
  }
}
