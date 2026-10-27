import { runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { DialogDescriptor, DialogProjectDescriptor } from "@xrf/ipc/types/xrf-dialog";
import { Nullable } from "@xrf/types";

import { mockCommandAnswer } from "@/core/operations/fixtures/command-operation.mocks";
import { mockHeldSessionRoot } from "@/core/session/fixtures/session-root.mocks";
import { DialogViewService } from "@/tools/dialogs-editor/services/dialog-view";
import { DialogsService } from "@/tools/dialogs-editor/services/dialogs";

/** A gamedata project holding one dialog file of one dialog, with two languages of text. */
export const MOCK_DIALOGS_PROJECT: DialogProjectDescriptor = {
  mode: "gamedata",
  roots: { asset: null, roots: [{ mode: "auto", path: "C:\\game" }] },
  dialogsPrefix: "configs\\gameplay",
  translationsPrefix: "configs\\text",
  isEditable: true,
  languages: ["eng", "rus"],
  textKeys: 24802,
  files: {
    "configs\\gameplay\\dialogs.xml": {
      physicalPath: "C:/game/configs/gameplay/dialogs.xml",
      isEditable: true,
      encoding: "windows-1251",
      dialogs: [{ id: "trader", phrases: 3, priority: null }],
    },
  },
  findings: [],
};

/** The one dialog that project holds, read in English. */
export const MOCK_DIALOG: DialogDescriptor = {
  logicalPath: "configs\\gameplay\\dialogs.xml",
  id: "trader",
  priority: null,
  elements: [],
  language: "eng",
  phrases: [
    {
      id: "0",
      textKey: "trader_hello",
      text: "Hello, stalker",
      isFinal: false,
      isInPhraseList: true,
      next: ["1"],
      elements: [],
    },
  ],
};

/**
 * Shows a dialog of a held project, one node inspected, as reading and picking it would, for a test whose setup cannot
 * wait for the read.
 *
 * @param dialogsService - The project's service, which is given the project.
 * @param viewService - The view to show the dialog.
 * @param dialog - What the read answered, or null for one that answered nothing.
 * @param nodeId - The node inspected, or none.
 */
export function mockShownDialog(
  dialogsService: DialogsService,
  viewService: DialogViewService,
  dialog: Nullable<DialogDescriptor>,
  nodeId: Nullable<string>
): void {
  const sessionId: string = "fixture-session";

  mockHeldSessionRoot(dialogsService.project, { revision: 0, sessionId, value: MOCK_DIALOGS_PROJECT });
  mockCommandAnswer(viewService["read"], AsyncState.ready(dialog));

  runInAction(() => {
    viewService["readFor"] = sessionId;
    viewService["view"] = {
      inspectedNodeId: nodeId,
      language: null,
      selection: dialog ? { id: dialog.id, logicalPath: dialog.logicalPath } : null,
      sessionId,
    };
  });
}
