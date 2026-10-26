import { describe, expect, it } from "@jest/globals";

import { CommandResult } from "@/core/ui/command-result/CommandResult";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("CommandResult", () => {
  it("leads with the outcome, which is what the command was run to learn", () => {
    const { getByText } = renderWithProviders(
      <CommandResult headline={"3 problem(s) found in 2 section(s)"} tone={"error"} stats={[]} />
    );

    expect(getByText("3 problem(s) found in 2 section(s)")).toBeInTheDocument();
  });

  it("renders each stat as a value with its label", () => {
    const { getByText } = renderWithProviders(
      <CommandResult
        headline={"Done"}
        tone={"success"}
        stats={[
          { label: "files", value: 12 },
          { label: "elapsed", value: "1.2 s" },
        ]}
      />
    );

    expect(getByText("12")).toBeInTheDocument();
    expect(getByText("files")).toBeInTheDocument();
    expect(getByText("1.2 s")).toBeInTheDocument();
  });

  it("takes its severity colour from the palette rather than a literal", () => {
    const { getByText } = renderWithProviders(<CommandResult headline={"Failed"} tone={"error"} stats={[]} />);

    const color: string = getComputedStyle(getByText("Failed")).color;

    // The components this replaced printed error headings in hardcoded `green`.
    expect(color).not.toBe("green");
    expect(color).toBeTruthy();
  });

  it("omits the findings divider when there are no findings", () => {
    const { container } = renderWithProviders(<CommandResult headline={"Done"} tone={"success"} stats={[]} />);

    expect(container.querySelector("hr")).not.toBeInTheDocument();
  });

  it("shows findings when given them", () => {
    const { getByText } = renderWithProviders(
      <CommandResult headline={"Done"} tone={"success"} stats={[]}>
        <div>finding rows</div>
      </CommandResult>
    );

    expect(getByText("finding rows")).toBeInTheDocument();
  });
});
