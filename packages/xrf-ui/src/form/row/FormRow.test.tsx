import { describe, expect, it } from "@jest/globals";

import { renderWithTheme } from "#/testing/render";

import { FormRow } from "./FormRow";

describe("FormRow", () => {
  it("supplies a control with the explicit ID, description, and invalid state", () => {
    const { getByRole } = renderWithTheme(
      <FormRow
        label={"Source"}
        controlId={"source-path"}
        description={"Directory to read"}
        error={"Path does not exist"}
      >
        {(props) => <input {...props} />}
      </FormRow>
    );
    const input: HTMLElement = getByRole("textbox", { name: "Source" });

    expect(input).toHaveAttribute("id", "source-path");
    expect(input).toHaveAccessibleDescription("Directory to read Path does not exist");
    expect(input).toHaveAttribute("aria-invalid", "true");
  });

  it("uses the fact when the validation error is empty", () => {
    const { getByRole } = renderWithTheme(
      <FormRow label={"Source"} error={""} fact={"Game installation"}>
        {(props) => <input {...props} />}
      </FormRow>
    );

    expect(getByRole("textbox", { name: "Source" })).toHaveAccessibleDescription("Game installation");
    expect(getByRole("textbox", { name: "Source" })).toHaveAttribute("aria-invalid", "false");
  });

  it("labels and explains the value", () => {
    const { getByText } = renderWithTheme(
      <FormRow label={"Configs directory"} description={"Directory of LTX files to validate"}>
        <input />
      </FormRow>
    );

    expect(getByText("Configs directory")).toBeInTheDocument();
    expect(getByText("Directory of LTX files to validate")).toBeInTheDocument();
  });

  it("marks the optional field rather than every required one", () => {
    const required = renderWithTheme(
      <FormRow label={"Source"} isRequired>
        <input />
      </FormRow>
    );

    expect(required.queryByText("Optional")).not.toBeInTheDocument();

    required.unmount();

    const optional = renderWithTheme(
      <FormRow label={"Source"} isRequired={false}>
        <input />
      </FormRow>
    );

    expect(optional.getByText("Optional")).toBeInTheDocument();
  });

  it("ties the label to the control, so the field is not announced as unlabelled", () => {
    const { getByLabelText } = renderWithTheme(
      <FormRow label={"Configs directory"} controlId={"configs-directory"}>
        <input id={"configs-directory"} />
      </FormRow>
    );

    expect(getByLabelText("Configs directory")).toBeInTheDocument();
  });

  it("shows a validation message when the value is wrong", () => {
    const { getByText } = renderWithTheme(
      <FormRow label={"Source"} error={"Path does not exist"}>
        <input />
      </FormRow>
    );

    expect(getByText("Path does not exist")).toBeInTheDocument();
  });

  it("omits the message when there is nothing wrong", () => {
    const { queryByText } = renderWithTheme(
      <FormRow label={"Source"}>
        <input />
      </FormRow>
    );

    expect(queryByText("Path does not exist")).not.toBeInTheDocument();
  });
});
