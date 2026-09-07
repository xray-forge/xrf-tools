import { describe, expect, it, jest } from "@jest/globals";

import { AssetRootFormRow } from "@/core/assets/components/AssetRootFormRow";
import { IPathField } from "@/core/ui/form/use-path-field";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

const INSTALLATION: string = "C:\\Games\\stalker";

/** A field as `useAssetRootField` produces one, with only what this row reads. */
function mockField(value: Nullable<string>, error: Nullable<string> = null): IPathField {
  return {
    clear: jest.fn(),
    commit: jest.fn(),
    error,
    isValid: true,
    pickRecent: jest.fn(),
    recents: { forget: jest.fn(), pick: jest.fn(), records: [] },
    select: jest.fn(),
    setValue: jest.fn(),
    value,
  } as unknown as IPathField;
}

describe("AssetRootFormRow", () => {
  it("says the root is optional, because reading only what you opened is an answer", () => {
    const { getByText } = renderWithProviders(<AssetRootFormRow field={mockField(null)} />);

    expect(getByText("Also search in")).toBeInTheDocument();
    expect(getByText("Optional")).toBeInTheDocument();
  });

  it("reports what the backend made of the directory", async () => {
    setMockInvokeResponses({
      ["plugin:assets|probe_root"]: { evidence: [], kind: "installation", mounts: 12 },
    });

    const { findByText, getByRole } = renderWithProviders(<AssetRootFormRow field={mockField(INSTALLATION)} />);

    // The whole reason the probe survived the settings section: a root named by hand is a guess until something
    // confirms it, and "installation" is the confirmation.
    expect(await findByText("Game installation, 12 sources")).toBeInTheDocument();
    expect(getByRole("textbox", { name: /Also search in/ })).toHaveAccessibleDescription(
      /Game installation, 12 sources/
    );
  });

  it("asks nothing about an empty field", () => {
    const { queryByText } = renderWithProviders(<AssetRootFormRow field={mockField(null)} />);

    expect(queryByText(/Game installation/)).not.toBeInTheDocument();
    expect(queryByText(/Game data/)).not.toBeInTheDocument();
  });

  it("says what is wrong instead of what it is, when the path is not there", async () => {
    setMockInvokeResponses({
      ["plugin:assets|probe_root"]: { evidence: ["textures"], kind: "root", mounts: 1 },
    });

    const { findByText, queryByText, getByRole } = renderWithProviders(
      <AssetRootFormRow field={mockField("Q:\\gone", "Path does not exist")} />
    );

    // An error wins the line, and nothing is probed: describing a directory that is not there would contradict it.
    expect(await findByText("Path does not exist")).toBeInTheDocument();
    expect(queryByText(/Game data/)).not.toBeInTheDocument();
    expect(getByRole("textbox", { name: /Also search in/ })).toHaveAccessibleDescription(/Path does not exist/);
    expect(getByRole("textbox", { name: /Also search in/ })).toHaveAttribute("aria-invalid", "true");
  });
});
