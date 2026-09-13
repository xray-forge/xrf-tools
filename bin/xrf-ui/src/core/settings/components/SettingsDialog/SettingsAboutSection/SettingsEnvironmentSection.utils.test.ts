import { describe, expect, it } from "@jest/globals";

import { HostInfo } from "@/core/bindings/types/xrf-app";
import { Nullable } from "@/lib/types/general";

import { IAboutRow } from "./about-row";
import { describeHost } from "./SettingsEnvironmentSection.utils";

function mockHost(overrides: Partial<HostInfo> = {}): HostInfo {
  return {
    tauriVersion: "2.11.5",
    webviewVersion: "140.0.3485.94",
    osName: "Windows",
    osVersion: "11",
    kernelVersion: "26200",
    arch: "x86_64",
    cpuCount: 16,
    physicalCoreCount: 8,
    totalMemory: 34_359_738_368,
    pid: 4242,
    ...overrides,
  };
}

function getValueOf(rows: Array<IAboutRow>, label: string): Nullable<string> {
  return rows.find((it: IAboutRow) => it.label === label)?.value ?? null;
}

describe("describeHost", () => {
  it("states the platform as one line and separates logical processors from cores", () => {
    const rows: Array<IAboutRow> = describeHost(mockHost());

    expect(getValueOf(rows, "Platform")).toBe("Windows 11");
    expect(getValueOf(rows, "Processors")).toBe("16 logical, 8 physical");
    expect(getValueOf(rows, "Memory")).toBe("32 GB");
  });

  it("drops a reading the operating system declined rather than stating an empty one", () => {
    const rows: Array<IAboutRow> = describeHost(
      mockHost({ webviewVersion: null, kernelVersion: null, osName: null, osVersion: null, physicalCoreCount: null })
    );

    expect(rows.map((it: IAboutRow) => it.label)).toEqual(["Tauri", "Architecture", "Processors", "Memory", "Process"]);
    // Both halves of the platform are absent, so the joined line is empty and is dropped rather than left blank.
    expect(getValueOf(rows, "Platform")).toBeNull();
    expect(getValueOf(rows, "Processors")).toBe("16 logical");
  });
});
