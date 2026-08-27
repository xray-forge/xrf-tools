import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { DialogInspectorPanel } from "@/applications/dialogs-editor/components/editor/DialogInspectorPanel";
import { DIALOG_NODE_ID } from "@/applications/dialogs-editor/lib";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { DialogDescriptor } from "@/core/bindings/types/xrf-dialog";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { createLoadable } from "@/lib/loadable";
import { Nullable } from "@/lib/types/general";

const DIALOG: DialogDescriptor = {
  logicalPath: "configs\\gameplay\\dialogs.xml",
  id: "trader",
  priority: -5,
  elements: [{ name: "precondition", kind: "precondition", value: "xr_conditions.actor_has_pda" }],
  language: "eng",
  phrases: [
    {
      id: "0",
      textKey: "trader_hello",
      text: "Hello, stalker",
      isFinal: true,
      isInPhraseList: true,
      next: ["1"],
      elements: [
        { name: "text", kind: "text", value: "trader_hello" },
        { name: "next", kind: "next", value: "1" },
        { name: "has_info", kind: "hasInfo", value: "met" },
        { name: "has_info", kind: "hasInfo", value: "armed" },
      ],
    },
    { id: "1", textKey: "trader_bye", text: null, isFinal: false, isInPhraseList: true, next: [], elements: [] },
  ],
};

/** Renders the panel against a service holding one dialog with `nodeId` inspected. */
function renderPanel(nodeId: Nullable<string>, dialog: Nullable<DialogDescriptor> = DIALOG): RenderResult {
  const container = mockContainer([DialogsService]);
  const service: DialogsService = container.get(DialogsService);

  service.dialog = createLoadable(dialog);
  service.inspectedNodeId = nodeId;

  return renderWithProviders(<DialogInspectorPanel />, { container });
}

describe("DialogInspectorPanel", () => {
  beforeEach(() => {
    setMockInvokeResponses({ ["plugin:dialogs|get_project"]: null });
  });

  it("invites a selection when nothing on the canvas is picked", () => {
    const { getByText } = renderPanel(null);

    expect(getByText("Nothing selected")).toBeInTheDocument();
  });

  it("leads with the line rather than with the phrases already on the canvas", () => {
    const { getAllByText, getByText, queryByText } = renderPanel("0");

    expect(getByText("Hello, stalker")).toBeInTheDocument();
    // The key stays visible under the line: it is what a save writes and what a missing translation
    // is reported against. Once only — the header owns it, so the property list does not repeat it.
    expect(getAllByText("trader_hello")).toHaveLength(1);
    expect(queryByText("text")).not.toBeInTheDocument();
    // The phrase it leads to is drawn on the canvas as a numbered edge, so it is not a property here.
    expect(queryByText("next")).not.toBeInTheDocument();
  });

  it("keeps a repeated element as repeated rows", () => {
    // Two info gates are two conditions. Folding them into one row would hide one.
    const { getAllByText } = renderPanel("0");

    expect(getAllByText("has_info")).toHaveLength(2);
  });

  it("marks a phrase whose key resolved to nothing", () => {
    const { getByText } = renderPanel("1");

    expect(getByText("untranslated")).toBeInTheDocument();
    expect(getByText("No text for this language.")).toBeInTheDocument();
  });

  it("describes the dialog itself when its root node is picked", () => {
    const { getByText } = renderPanel(DIALOG_NODE_ID);

    expect(getByText("Dialog")).toBeInTheDocument();
    expect(getByText("priority -5")).toBeInTheDocument();
    expect(getByText("precondition")).toBeInTheDocument();
    expect(getByText("2 phrases · eng")).toBeInTheDocument();
    // A dialog gates a conversation; a phrase carries elements. The band says which it is looking at.
    expect(getByText("Conditions")).toBeInTheDocument();
  });

  it("names the subject and its element band per node kind", () => {
    const { getByText } = renderPanel("0");

    expect(getByText("Phrase")).toBeInTheDocument();
    expect(getByText("Elements")).toBeInTheDocument();
  });

  it("says so when the inspected phrase is no longer declared", () => {
    // Reachable while a language switch is in flight: the canvas keeps its selection across the
    // re-fetch, so the panel can be pointed at a phrase the incoming dialog does not hold.
    const { getByText } = renderPanel("gone");

    expect(getByText("Phrase is gone")).toBeInTheDocument();
  });
});
