import { describe, expect, it } from "@jest/globals";

import * as generated from "@/core/ipc/types/xrf-app";

import { findJobKind, JOB_KINDS } from "./job-kinds";

/** Every job kind the backend declares, as the generated `<DOMAIN>_JOB_<OPERATION>` constants carry them. */
const DECLARED_KINDS: Array<string> = Object.entries(generated)
  .filter(([name, value]) => /^[A-Z][A-Z_]*_JOB_[A-Z_]+$/.test(name) && typeof value === "string")
  .map(([, value]) => value as string);

describe("JOB_KINDS", () => {
  it("reads the generated job kinds", () => {
    expect(DECLARED_KINDS).toContain(generated.ARCHIVES_JOB_PACK);
  });

  it("describes every job kind the backend declares, under its own kind", () => {
    for (const kind of DECLARED_KINDS) {
      expect(findJobKind(kind)?.kind).toBe(kind);
    }
  });

  it("describes no kind the backend does not declare", () => {
    expect(Object.keys(JOB_KINDS).sort()).toEqual([...DECLARED_KINDS].sort());
  });
});
