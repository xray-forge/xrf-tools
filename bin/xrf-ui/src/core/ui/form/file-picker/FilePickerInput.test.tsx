import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { FilePickerInput } from "@/core/ui/form/file-picker/FilePickerInput";
import { IPathFieldRecents, IPathRecord } from "@/core/ui/form/path-recents";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("FilePickerInput", () => {
  it("shows the picked path", () => {
    const { getByDisplayValue } = renderWithProviders(
      <FilePickerInput value={"C:\\gamedata\\config"} onSelect={jest.fn()} />
    );

    expect(getByDisplayValue("C:\\gamedata\\config")).toBeInTheDocument();
  });

  it("says so when nothing is picked yet", () => {
    const { getByPlaceholderText } = renderWithProviders(<FilePickerInput onSelect={jest.fn()} />);

    expect(getByPlaceholderText("Not selected")).toBeInTheDocument();
  });

  it("leaves the field alone so the path can be selected and copied", async () => {
    const onSelect = jest.fn();

    const { getByRole } = renderWithProviders(<FilePickerInput value={"C:\\gamedata"} onSelect={onSelect} />);

    await userEvent.click(getByRole("textbox"));

    // Clicking the text used to reopen the dialog, which made the value impossible to select by hand.
    expect(onSelect).not.toHaveBeenCalled();
  });

  it("selects from the browse button", async () => {
    const onSelect = jest.fn();

    const { getByLabelText } = renderWithProviders(<FilePickerInput value={"C:\\gamedata"} onSelect={onSelect} />);

    await userEvent.click(getByLabelText("Browse"));

    expect(onSelect).toHaveBeenCalledTimes(1);
  });

  it("takes a typed or pasted path when the caller accepts one", async () => {
    const onChange = jest.fn();

    const { getByRole } = renderWithProviders(<FilePickerInput onChange={onChange} onSelect={jest.fn()} />);

    await userEvent.type(getByRole("textbox"), "D:");

    expect(onChange).toHaveBeenCalled();
  });

  it("stays read only when the caller has no way to accept typed input", async () => {
    const { getByRole } = renderWithProviders(<FilePickerInput value={"C:\\gamedata"} onSelect={jest.fn()} />);

    expect(getByRole("textbox")).toHaveAttribute("readonly");
  });

  it("clears without also opening the dialog", async () => {
    const onSelect = jest.fn();
    const onClear = jest.fn();

    const { getByLabelText } = renderWithProviders(
      <FilePickerInput value={"C:\\gamedata"} onSelect={onSelect} onClear={onClear} />
    );

    await userEvent.click(getByLabelText("Clear"));

    // The clear control sits inside the field, whose click opens the picker. Without the guard in the
    // component, clearing would immediately reopen the dialog it just cleared for.
    expect(onClear).toHaveBeenCalledTimes(1);
    expect(onSelect).not.toHaveBeenCalled();
  });

  it("offers nothing to clear when there is no value", () => {
    const { queryByLabelText } = renderWithProviders(<FilePickerInput onSelect={jest.fn()} onClear={jest.fn()} />);

    expect(queryByLabelText("Clear")).not.toBeInTheDocument();
  });

  it("omits the clear control when the caller does not support clearing", () => {
    const { queryByLabelText } = renderWithProviders(<FilePickerInput value={"C:\\gamedata"} onSelect={jest.fn()} />);

    expect(queryByLabelText("Clear")).not.toBeInTheDocument();
  });

  it("does not select while disabled", () => {
    const { getByLabelText, getByRole } = renderWithProviders(
      <FilePickerInput value={"C:\\gamedata"} isDisabled onSelect={jest.fn()} />
    );

    expect(getByLabelText("Browse")).toBeDisabled();
    expect(getByRole("textbox")).toBeDisabled();
  });

  it("describes what the path is for", () => {
    const { getByText } = renderWithProviders(
      <FilePickerInput label={"Configs"} description={"Directory of LTX files to validate"} onSelect={jest.fn()} />
    );

    expect(getByText("Directory of LTX files to validate")).toBeInTheDocument();
  });

  describe("history", () => {
    const RECORDS: Array<IPathRecord> = [
      { at: Date.now(), path: "C:\\Projects\\stalker\\gamedata-anomaly\\configs" },
      { at: Date.now() - 60_000, path: "C:\\Projects\\stalker\\gamedata\\configs" },
    ];

    /** A history as a field offers one, with the actions this control is expected to call. */
    function offered(records: Array<IPathRecord>, actions: Partial<IPathFieldRecents> = {}): IPathFieldRecents {
      return { forget: jest.fn(), pick: jest.fn(), records, ...actions };
    }

    it("offers nothing at all when there is no history", () => {
      const { queryByLabelText } = renderWithProviders(<FilePickerInput onSelect={jest.fn()} recents={offered([])} />);

      // A button that opens an empty menu reads as broken rather than as unused.
      expect(queryByLabelText("Recent paths")).not.toBeInTheDocument();
    });

    it("offers each remembered path", async () => {
      const { getByLabelText, getByText } = renderWithProviders(
        <FilePickerInput
          onSelect={jest.fn()}
          recents={offered(RECORDS)}
        />
      );

      await userEvent.click(getByLabelText("Recent paths"));

      // Both fit at this width, so both are shown whole. Two directories called `configs` are told apart by what is
      // above them, which is why a row that has to be cut is cut from the front.
      expect(getByText(RECORDS[0].path)).toBeInTheDocument();
      expect(getByText(RECORDS[1].path)).toBeInTheDocument();
    });

    it("cuts a path too long for a row from the front, keeping the tail that identifies it", async () => {
      const long: string = "D:\\archive\\mods\\a-very-long-modification-directory-name\\gamedata\\configs";

      const { getByLabelText, getByText } = renderWithProviders(
        <FilePickerInput
          onSelect={jest.fn()}
          recents={offered([{ at: Date.now(), path: long }])}
        />
      );

      await userEvent.click(getByLabelText("Recent paths"));

      const row: HTMLElement = getByText(/configs$/);

      expect(row.textContent?.startsWith("…")).toBe(true);
      expect(row.textContent).toContain("gamedata\\configs");
      // The whole path is still reachable, so nothing is lost by shortening the row.
      expect(getByLabelText(`Forget ${long}`)).toBeInTheDocument();
    });

    it("reports the path that was chosen", async () => {
      const onPickRecent = jest.fn();

      const { getByLabelText, getByText } = renderWithProviders(
        <FilePickerInput
          onSelect={jest.fn()}
          recents={offered(RECORDS, { pick: onPickRecent })}
        />
      );

      await userEvent.click(getByLabelText("Recent paths"));
      await userEvent.click(getByText(RECORDS[1].path));

      expect(onPickRecent).toHaveBeenCalledWith("C:\\Projects\\stalker\\gamedata\\configs");
    });

    it("forgets one entry without choosing it", async () => {
      const onPickRecent = jest.fn();
      const onForgetRecent = jest.fn();

      const { getByLabelText } = renderWithProviders(
        <FilePickerInput
          onSelect={jest.fn()}
          recents={offered(RECORDS, { forget: onForgetRecent, pick: onPickRecent })}
        />
      );

      await userEvent.click(getByLabelText("Recent paths"));
      await userEvent.click(getByLabelText("Forget C:\\Projects\\stalker\\gamedata\\configs"));

      expect(onForgetRecent).toHaveBeenCalledWith("C:\\Projects\\stalker\\gamedata\\configs");
      // Otherwise the row underneath answers the same click and the field fills with what was just discarded.
      expect(onPickRecent).not.toHaveBeenCalled();
    });
  });
});
