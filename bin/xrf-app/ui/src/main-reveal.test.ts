import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { ESystemCommand } from "@xrf/ipc/commands/system";
import { mockInvoke } from "@xrf/ipc/testing";

/**
 * Runs the inlined script as `index.html` does, once. Required rather than imported, since a script with nothing to
 * export is not a module to the compiler.
 */
function runRevealScript(): void {
  jest.isolateModules(() => {
    jest.requireActual("@/main-reveal");
  });
}

describe("main-reveal", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
    Object.assign(window, { __TAURI_INTERNALS__: { invoke: mockInvoke } });
  });

  afterEach(() => {
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");
  });

  it("reveals the window through the command the generated bindings name", () => {
    runRevealScript();

    // The one call the generated bindings cannot make, spelled out by hand: checked here against them, so a renamed
    // command fails this test rather than leaving every launch to the backend's timeout.
    expect(mockInvoke.mock.calls).toEqual([[ESystemCommand.REVEAL_WINDOW]]);
  });

  it("does nothing outside the desktop runtime", () => {
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");

    expect(runRevealScript).not.toThrow();
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
