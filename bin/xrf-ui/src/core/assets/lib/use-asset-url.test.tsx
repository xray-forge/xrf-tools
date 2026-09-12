import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { ReactElement } from "react";

import { useAssetUrl } from "@/core/assets/lib/use-asset-url";
import { AssetService } from "@/core/assets/services";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

const KEY: string = "texture-preview";

function Probe({ bytes }: { bytes: Nullable<ArrayBuffer> }): ReactElement {
  const url: Nullable<string> = useAssetUrl(KEY, bytes);

  return <div data-testid={"probe"}>{url ?? "none"}</div>;
}

describe("useAssetUrl", () => {
  let created: number = 0;
  let revoked: Array<string> = [];

  beforeEach(() => {
    created = 0;
    revoked = [];

    jest.spyOn(URL, "createObjectURL").mockImplementation(() => `blob:asset/${(created += 1)}`);
    jest.spyOn(URL, "revokeObjectURL").mockImplementation((url: string) => {
      revoked.push(url);
    });
  });

  it("shows the bytes it is given", () => {
    const { getByTestId }: RenderResult = renderWithProviders(<Probe bytes={new ArrayBuffer(4)} />, {
      container: mockContainer([AssetService]),
    });

    expect(getByTestId("probe").textContent).toBe("blob:asset/1");
    expect(revoked).toEqual([]);
  });

  it("releases the url when the picture goes away", () => {
    const container: Container = mockContainer([AssetService]);
    const { rerender, getByTestId }: RenderResult = renderWithProviders(<Probe bytes={new ArrayBuffer(4)} />, {
      container,
    });

    rerender(<Probe bytes={null} />);

    expect(getByTestId("probe").textContent).toBe("none");
    expect(revoked).toEqual(["blob:asset/1"]);
    expect(container.get(AssetService).heldCount).toBe(0);
  });

  it("releases the url when the preview is left behind", () => {
    const container: Container = mockContainer([AssetService]);
    const { unmount }: RenderResult = renderWithProviders(<Probe bytes={new ArrayBuffer(4)} />, { container });

    unmount();

    // Leaving an editor unmounts its preview without deactivating the application that owns the service, so a url
    // nobody releases here outlives every session until the window closes.
    expect(revoked).toEqual(["blob:asset/1"]);
    expect(container.get(AssetService).heldCount).toBe(0);
  });

  it("keeps the replaced url alive until its replacement exists", () => {
    const { rerender }: RenderResult = renderWithProviders(<Probe bytes={new ArrayBuffer(4)} />, {
      container: mockContainer([AssetService]),
    });

    rerender(<Probe bytes={new ArrayBuffer(8)} />);

    // Revoking before creating leaves whatever is still rendering the old url pointing at nothing, so the replacement
    // is made first and the old one revoked after it.
    expect(created).toBe(2);
    expect(revoked).toEqual(["blob:asset/1"]);
  });
});
