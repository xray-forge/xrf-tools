import { describe, expect, it, jest } from "@jest/globals";
import { render, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { PackerConfirmSummary } from "@/applications/archives-packer/components/packing/PackerConfirmSummary";
import { FALLBACK_PACK_CONFIG } from "@/applications/archives-packer/lib/pack-config";
import { ArchivePackConfig } from "@/core/bindings/types/xrf-pack";

const CONFIG: ArchivePackConfig = {
  ...FALLBACK_PACK_CONFIG,
  source: "C:\\work\\gamedata",
  destination: "C:\\work\\db",
  name: "gamedata",
};

function renderSummary(publishedVolumes: Array<string>, onForceChange: (isForced: boolean) => void): RenderResult {
  return render(
    <PackerConfirmSummary
      config={CONFIG}
      publishedVolumes={publishedVolumes}
      isForced={false}
      onForceChange={onForceChange}
    />
  );
}

describe("PackerConfirmSummary", () => {
  it("says nothing about replacing an archive when the destination holds none", () => {
    // A warning shown every time is a warning nobody reads, and it would be untrue besides: a pack into a destination
    // holding no set of this name replaces nothing.
    const { queryByRole, queryByText } = renderSummary([], jest.fn());

    expect(queryByRole("checkbox")).toBeNull();
    expect(queryByText(/already holds/)).toBeNull();
  });

  it("asks before replacing volumes the destination already holds", async () => {
    const onForceChange: (isForced: boolean) => void = jest.fn();
    const { getByRole, getByText } = renderSummary(
      ["C:\\work\\db\\gamedata.db0", "C:\\work\\db\\gamedata.db1"],
      onForceChange
    );

    expect(getByText(/already holds 2 volume\(s\) of gamedata\.db/)).toBeDefined();

    await userEvent.click(getByRole("checkbox"));

    expect(onForceChange).toHaveBeenCalledWith(true);
  });
});
