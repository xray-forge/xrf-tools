import { describe, expect, it } from "@jest/globals";
import { EnvFlagReading } from "@xrf/ipc/types/xrf-env-flags";

import { describeVariables } from "./SettingsVariablesSection.utils";

function mockReading(overrides: Partial<EnvFlagReading> = {}): EnvFlagReading {
  return {
    name: "XRF_RENDER_BACKEND",
    kind: "runtime",
    description: "The graphics API the renderer starts on.",
    default: "the settings' choice",
    value: null,
    applied: null,
    ...overrides,
  };
}

describe("describeVariables", () => {
  it("states only the variables set", () => {
    expect(describeVariables([mockReading(), mockReading({ name: "RUST_LOG" })])).toEqual([]);
  });

  it("states a set variable's value as given", () => {
    expect(describeVariables([mockReading({ value: "Vulkan", applied: "vulkan" })])).toEqual([
      { label: "XRF_RENDER_BACKEND", value: "Vulkan" },
    ]);
  });

  it("says which default applies where a variable does not take its value", () => {
    expect(describeVariables([mockReading({ value: "dx12" })])).toEqual([
      { label: "XRF_RENDER_BACKEND", value: "dx12 (not valid, the default applies: the settings' choice)" },
    ]);
  });
});
