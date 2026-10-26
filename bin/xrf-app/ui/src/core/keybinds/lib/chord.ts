import { Optional } from "@xrf/types";

import { isApplePlatform } from "@/lib/env";

/**
 * Modifier spelled in a chord, resolved per platform when the chord is matched.
 */
const MOD: string = "mod";

/** Modifiers in the one order a canonical chord spells them, so two spellings of one chord compare equal. */
const MODIFIER_ORDER: ReadonlyArray<string> = [MOD, "ctrl", "meta", "alt", "shift"];

/**
 * Key tokens whose canonical spelling is not their own lowercase form.
 */
const NAMED_KEYS: ReadonlyMap<string, string> = new Map([
  ["arrowdown", "ArrowDown"],
  ["arrowleft", "ArrowLeft"],
  ["arrowright", "ArrowRight"],
  ["arrowup", "ArrowUp"],
  ["backspace", "Backspace"],
  ["delete", "Delete"],
  ["end", "End"],
  ["enter", "Enter"],
  ["escape", "Escape"],
  ["home", "Home"],
  ["insert", "Insert"],
  ["pagedown", "PageDown"],
  ["pageup", "PageUp"],
  ["space", " "],
  ["tab", "Tab"],
]);

/** How a named key is written in a shortcuts listing, where a raw `" "` would be invisible. */
const KEY_LABELS: ReadonlyMap<string, string> = new Map([[" ", "Space"]]);

const FUNCTION_KEY_PATTERN: RegExp = /^f([1-9]|1[0-9]|2[0-4])$/;

/** One parsed chord, with every spelling variation already resolved. */
export interface IKeyChord {
  /** `event.key` as this chord expects it: a lowercase character, or a canonical key name. */
  readonly key: string;
  readonly hasMod: boolean;
  readonly hasCtrl: boolean;
  readonly hasMeta: boolean;
  readonly hasAlt: boolean;
  readonly hasShift: boolean;
}

/**
 * Parses an authored chord such as `mod+shift+p`, `/` or `F1`.
 *
 * @param chord - Chord as authored on a command declaration.
 * @returns The parsed chord.
 */
export function parseChord(chord: string): IKeyChord {
  const tokens: Array<string> = chord
    .split("+")
    .map((token: string) => token.trim().toLowerCase())
    .filter((token: string) => token.length > 0);

  if (tokens.length === 0) {
    throw new Error(`Chord '${chord}' names no key.`);
  }

  const modifiers: Set<string> = new Set();
  const keys: Array<string> = [];

  for (const token of tokens) {
    if (MODIFIER_ORDER.includes(token)) {
      modifiers.add(token);
    } else {
      keys.push(token);
    }
  }

  if (keys.length !== 1) {
    throw new Error(`Chord '${chord}' must name exactly one key, found ${keys.length}.`);
  }

  const [key] = keys;

  return {
    hasAlt: modifiers.has("alt"),
    hasCtrl: modifiers.has("ctrl"),
    hasMeta: modifiers.has("meta"),
    hasMod: modifiers.has(MOD),
    hasShift: modifiers.has("shift"),
    key: toCanonicalKey(key),
  };
}

/**
 * @param key - Key token as authored, already lowercased.
 * @returns The spelling `event.key` uses.
 */
function toCanonicalKey(key: string): string {
  const named: Optional<string> = NAMED_KEYS.get(key);

  if (named) {
    return named;
  }

  return FUNCTION_KEY_PATTERN.test(key) ? key.toUpperCase() : key;
}

/**
 * The one string two equal chords share, whatever order they were authored in.
 *
 * @param chord - Parsed chord.
 * @returns Its canonical form.
 */
export function toChordKey(chord: IKeyChord): string {
  const modifiers: Array<string> = MODIFIER_ORDER.filter((modifier: string) => {
    switch (modifier) {
      case MOD:
        return chord.hasMod;
      case "ctrl":
        return chord.hasCtrl;
      case "meta":
        return chord.hasMeta;
      case "alt":
        return chord.hasAlt;
      default:
        return chord.hasShift;
    }
  });

  return [...modifiers, chord.key].join("+");
}

/**
 * How a chord is written for a person.
 *
 * @param chord - Parsed chord.
 * @returns A display string such as `Ctrl + Shift + P`.
 */
export function formatChord(chord: IKeyChord): string {
  const isApple: boolean = isApplePlatform();
  const parts: Array<string> = [];

  if (chord.hasMod) {
    parts.push(isApple ? "Cmd" : "Ctrl");
  }

  if (chord.hasCtrl) {
    parts.push("Ctrl");
  }

  if (chord.hasMeta) {
    parts.push(isApple ? "Cmd" : "Win");
  }

  if (chord.hasAlt) {
    parts.push(isApple ? "Option" : "Alt");
  }

  if (chord.hasShift) {
    parts.push("Shift");
  }

  const label: string = KEY_LABELS.get(chord.key) ?? chord.key;

  parts.push(label.length === 1 ? label.toUpperCase() : label);

  return parts.join(" + ");
}

/**
 * Whether a chord is suppressed while a text field has the key.
 *
 * Derived from the chord rather than declared per command: a bare printable character is what someone is typing, and
 * anything carrying a modifier, or a named key such as `Escape` or `F1`, is not. This is the rule the hand-written
 * launcher hotkey already applied to `/` and `mod+k`.
 *
 * @param chord - Parsed chord.
 * @returns Whether text entry keeps the key.
 */
export function isSuppressedInTextEntry(chord: IKeyChord): boolean {
  const isModified: boolean = chord.hasMod || chord.hasCtrl || chord.hasMeta || chord.hasAlt;

  return !isModified && chord.key.length === 1;
}

/**
 * Whether a key event is this chord.
 *
 * @param event - Keyboard event being dispatched.
 * @param chord - Parsed chord to test.
 * @returns Whether they match, modifiers included.
 */
export function matchesChord(event: KeyboardEvent, chord: IKeyChord): boolean {
  const isApple: boolean = isApplePlatform();
  const expectsCtrl: boolean = chord.hasCtrl || (chord.hasMod && !isApple);
  const expectsMeta: boolean = chord.hasMeta || (chord.hasMod && isApple);

  if (event.ctrlKey !== expectsCtrl || event.metaKey !== expectsMeta) {
    return false;
  }

  if (event.altKey !== chord.hasAlt || event.shiftKey !== chord.hasShift) {
    return false;
  }

  return (event.key.length === 1 ? event.key.toLowerCase() : event.key) === chord.key;
}
