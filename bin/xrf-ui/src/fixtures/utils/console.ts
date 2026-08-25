import { afterEach, beforeEach, jest } from "@jest/globals";

import { noop } from "@/lib/callbacks/noop";
import { Nullable } from "@/lib/types/general";

export type TConsoleChannel = "debug" | "error" | "info" | "log" | "warn";

/**
 * Silences one console channel for the enclosing block.
 *
 * @param channel - Console method to silence for each test in the enclosing block.
 */
export function muteConsole(channel: TConsoleChannel): void {
  let muted: Nullable<jest.SpiedFunction<Console[TConsoleChannel]>> = null;

  beforeEach(() => {
    muted = jest.spyOn(console, channel).mockImplementation(noop);
  });

  afterEach(() => {
    muted?.mockRestore();
    muted = null;
  });
}
