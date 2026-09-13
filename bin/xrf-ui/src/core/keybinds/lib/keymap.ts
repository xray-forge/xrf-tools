import { ICommandDescriptor } from "@/core/commands";
import { IKeyChord, isSuppressedInTextEntry, matchesChord, parseChord, toChordKey } from "@/core/keybinds/lib/chord";
import { Nullable, Optional } from "@/lib/types/general";

/** One chord bound to one command. */
export interface IKeybinding {
  readonly chord: IKeyChord;
  readonly command: ICommandDescriptor;
}

/**
 * The bindings reachable from where the application currently is.
 */
export interface IKeymap {
  /** Fires wherever focus is. */
  readonly anywhere: ReadonlyArray<IKeybinding>;
  /** Bare printable chords, which text entry keeps. */
  readonly outsideTextEntry: ReadonlyArray<IKeybinding>;
}

/** Chord claimed by more than one command among a reachable set. */
export interface IChordConflict {
  /** Canonical chord, so two spellings of one chord report once. */
  readonly chord: string;
  readonly commandIds: ReadonlyArray<string>;
}

/**
 * Builds the lookup a dispatcher scans.
 *
 * @param commands - Commands reachable right now.
 * @param getChords - Resolves a command's effective chords, override included.
 * @returns The keymap, partitioned by suppression.
 */
export function buildKeymap(
  commands: ReadonlyArray<ICommandDescriptor>,
  getChords: (command: ICommandDescriptor) => ReadonlyArray<string>
): IKeymap {
  const anywhere: Array<IKeybinding> = [];
  const outsideTextEntry: Array<IKeybinding> = [];
  const claimedChords: Map<string, Set<string>> = new Map();

  for (const command of commands) {
    for (const authored of getChords(command)) {
      const chord: IKeyChord = parseChord(authored);
      const canonical: string = toChordKey(chord);
      // Keyed by id, the identity the bus and the overlay already use, so a command reaching this twice - listed by
      // its application and by a core set it composes - binds once however it was reached.
      const claimed: Set<string> = claimedChords.get(command.id) ?? new Set();

      if (claimed.has(canonical)) {
        continue;
      }

      claimed.add(canonical);
      claimedChords.set(command.id, claimed);
      (isSuppressedInTextEntry(chord) ? outsideTextEntry : anywhere).push({ chord, command });
    }
  }

  return { anywhere, outsideTextEntry };
}

/**
 * Finds the command a key event invokes.
 *
 * @param keymap - Bindings reachable right now.
 * @param event - Key event being dispatched.
 * @param isTextEntry - Whether text entry has the key.
 * @returns The command to run, or null.
 */
export function resolveKeybinding(
  keymap: IKeymap,
  event: KeyboardEvent,
  isTextEntry: boolean
): Nullable<ICommandDescriptor> {
  // Scanned in place rather than concatenated: this runs on every key a person types, including into a field.
  const matched: Nullable<ICommandDescriptor> = findBoundCommand(keymap.anywhere, event);

  if (matched || isTextEntry) {
    return matched;
  }

  return findBoundCommand(keymap.outsideTextEntry, event);
}

/**
 * @param bindings - Bindings to scan.
 * @param event - Key event being dispatched.
 * @returns The first command whose chord the event is, or null.
 */
function findBoundCommand(bindings: ReadonlyArray<IKeybinding>, event: KeyboardEvent): Nullable<ICommandDescriptor> {
  for (const binding of bindings) {
    if (matchesChord(event, binding.chord)) {
      return binding.command;
    }
  }

  return null;
}

/**
 * Chords claimed by more than one command in a set that can be active at once.
 *
 * @param commands - Commands reachable together.
 * @returns One entry per contested chord, in canonical spelling.
 */
export function findChordConflicts(commands: ReadonlyArray<ICommandDescriptor>): Array<IChordConflict> {
  const byChord: Map<string, Set<string>> = new Map();

  for (const command of commands) {
    for (const authored of command.chords) {
      const chord: string = toChordKey(parseChord(authored));
      const owners: Optional<Set<string>> = byChord.get(chord);

      if (owners) {
        owners.add(command.id);
      } else {
        byChord.set(chord, new Set([command.id]));
      }
    }
  }

  return [...byChord.entries()]
    .filter(([, owners]: [string, Set<string>]) => owners.size > 1)
    .map(([chord, owners]: [string, Set<string>]) => ({ chord, commandIds: [...owners] }));
}
