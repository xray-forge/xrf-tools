import { describe, expect, it } from "@jest/globals";

import { EArchiveSubject } from "@/core/archive/lib/subject";
import { ArchiveSubject } from "@/core/ipc/types/xrf-app";

describe("EArchiveSubject", () => {
  it("declares exactly the discriminants the generated union carries", () => {
    const mirrored: Readonly<Record<ArchiveSubject["kind"], EArchiveSubject>> = {
      [EArchiveSubject.VOLUMES]: EArchiveSubject.VOLUMES,
      [EArchiveSubject.WORLD]: EArchiveSubject.WORLD,
    };

    expect(Object.values(mirrored)).toEqual(Object.values(EArchiveSubject));
  });
});
