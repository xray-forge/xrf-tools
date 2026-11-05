import { describe, expect, it } from "@jest/globals";

import { formatChord, isSuppressedInTextEntry, matchesChord, parseChord, toChordKey } from "./chord";

function mockKeyEvent(init: KeyboardEventInit): KeyboardEvent {
  return new KeyboardEvent("keydown", { altKey: false, ctrlKey: false, metaKey: false, shiftKey: false, ...init });
}

describe("parseChord", () => {
  it("reads a bare character, a modified one and a named key", () => {
    expect(parseChord("/")).toEqual({
      hasAlt: false,
      hasCtrl: false,
      hasMeta: false,
      hasMod: false,
      hasShift: false,
      key: "/",
    });

    expect(parseChord("mod+k").hasMod).toBe(true);
    expect(parseChord("mod+k").key).toBe("k");
    expect(parseChord("Escape").key).toBe("Escape");
  });

  it("canonicalizes spelling, so one chord is one chord however it was written", () => {
    expect(toChordKey(parseChord("mod+shift+p"))).toBe(toChordKey(parseChord("Shift + MOD + P")));
    expect(parseChord("f5").key).toBe("F5");
    expect(parseChord("arrowdown").key).toBe("ArrowDown");
    expect(parseChord("space").key).toBe(" ");
  });

  it.each([
    ["", "names no key"],
    ["mod+", "exactly one key"],
    ["mod", "exactly one key"],
    ["mod+k+j", "exactly one key"],
  ])("refuses '%s' rather than binding a shortcut that never fires", (chord: string, reason: string) => {
    expect(() => parseChord(chord)).toThrow(reason);
  });
});

describe("isSuppressedInTextEntry", () => {
  it("keeps bare characters out of fields and lets everything else through", () => {
    expect(isSuppressedInTextEntry(parseChord("/"))).toBe(true);
    // Shift is how a capital is typed, so it does not make a character a shortcut.
    expect(isSuppressedInTextEntry(parseChord("shift+a"))).toBe(true);

    expect(isSuppressedInTextEntry(parseChord("mod+k"))).toBe(false);
    expect(isSuppressedInTextEntry(parseChord("F1"))).toBe(false);
    expect(isSuppressedInTextEntry(parseChord("Escape"))).toBe(false);
  });
});

describe("matchesChord", () => {
  it("resolves mod to the platform accelerator", () => {
    expect(matchesChord(mockKeyEvent({ ctrlKey: true, key: "k" }), parseChord("mod+k"))).toBe(true);
    expect(matchesChord(mockKeyEvent({ key: "k", metaKey: true }), parseChord("mod+k"))).toBe(false);
  });

  // A Ukrainian layout sends `п` for the key a US one labels G: matched by the letter, no letter chord reached it.
  it("reads a key a non-Latin layout prints as its place on a US keyboard", () => {
    expect(matchesChord(mockKeyEvent({ code: "KeyG", ctrlKey: true, key: "п" }), parseChord("mod+g"))).toBe(true);
    expect(matchesChord(mockKeyEvent({ code: "KeyH", key: "р" }), parseChord("h"))).toBe(true);
    expect(matchesChord(mockKeyEvent({ code: "KeyG", ctrlKey: true, key: "п" }), parseChord("mod+h"))).toBe(false);
  });

  // Only a letter falls back to its place: a symbol is what a chord names, whichever key a layout prints it on.
  it("reads a symbol as itself, never as the digit key it sits on", () => {
    expect(matchesChord(mockKeyEvent({ code: "Digit1", key: "!", shiftKey: true }), parseChord("shift+!"))).toBe(true);
    expect(matchesChord(mockKeyEvent({ code: "Digit1", key: "!", shiftKey: true }), parseChord("shift+1"))).toBe(false);
    expect(matchesChord(mockKeyEvent({ code: "Digit1", ctrlKey: true, key: "&" }), parseChord("mod+1"))).toBe(false);
  });

  // AZERTY prints Z where a US keyboard has W: the chord follows the letter a Latin layout prints, not the place.
  it("keeps a Latin layout's own letters", () => {
    expect(matchesChord(mockKeyEvent({ code: "KeyW", ctrlKey: true, key: "z" }), parseChord("mod+z"))).toBe(true);
    expect(matchesChord(mockKeyEvent({ code: "KeyW", ctrlKey: true, key: "z" }), parseChord("mod+w"))).toBe(false);
  });

  it("compares characters case insensitively but names exactly", () => {
    expect(matchesChord(mockKeyEvent({ ctrlKey: true, key: "K" }), parseChord("mod+k"))).toBe(true);
    expect(matchesChord(mockKeyEvent({ key: "F1" }), parseChord("F1"))).toBe(true);
    expect(matchesChord(mockKeyEvent({ key: "F2" }), parseChord("F1"))).toBe(false);
  });

  it("refuses a modifier the chord did not ask for, so mod+k is not k", () => {
    expect(matchesChord(mockKeyEvent({ key: "k" }), parseChord("mod+k"))).toBe(false);
    expect(matchesChord(mockKeyEvent({ ctrlKey: true, key: "k" }), parseChord("k"))).toBe(false);
    expect(matchesChord(mockKeyEvent({ altKey: true, ctrlKey: true, key: "k" }), parseChord("mod+k"))).toBe(false);
  });
});

describe("formatChord", () => {
  it("writes a chord the way a listing shows it", () => {
    expect(formatChord(parseChord("mod+k"))).toBe("Ctrl + K");
    expect(formatChord(parseChord("F1"))).toBe("F1");
    expect(formatChord(parseChord("/"))).toBe("/");
    expect(formatChord(parseChord("space"))).toBe("Space");
  });
});
