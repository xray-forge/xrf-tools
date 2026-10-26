import { describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ReactElement, useState } from "react";

import { renderWithProviders } from "@/fixtures/utils/render";

import { StringListFormRow } from "./StringListFormRow";

function StringListExample({ isDisabled = false }: { isDisabled?: boolean }): ReactElement {
  const [values, setValues] = useState<Array<string>>(["*.txt"]);

  return (
    <StringListFormRow
      label={"Excluded extensions"}
      description={"Patterns to leave out"}
      values={values}
      addLabel={"Add pattern"}
      emptyLabel={"No patterns"}
      isDisabled={isDisabled}
      onChange={setValues}
    />
  );
}

describe("StringListFormRow", () => {
  it("names its fields and preserves edits while adding and removing entries", async () => {
    const { getByRole, queryByRole } = renderWithProviders(<StringListExample />);

    await userEvent.click(getByRole("button", { name: "Add pattern" }));
    await userEvent.type(getByRole("textbox", { name: "Add pattern 2" }), "*.log");
    await userEvent.click(getByRole("button", { name: "Remove *.txt" }));

    expect(getByRole("textbox", { name: "Add pattern 1" })).toHaveValue("*.log");
    expect(queryByRole("textbox", { name: "Add pattern 2" })).not.toBeInTheDocument();
  });

  it("disables editing, addition and removal together", () => {
    const { getByRole } = renderWithProviders(<StringListExample isDisabled />);

    expect(getByRole("textbox", { name: "Add pattern 1" })).toBeDisabled();
    expect(getByRole("button", { name: "Add pattern" })).toBeDisabled();
    expect(getByRole("button", { name: "Remove *.txt" })).toBeDisabled();
  });
});
