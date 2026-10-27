import { useInjection } from "@wirestate/react";
import { DialogDescriptor, DialogElementDescriptor, DialogPhraseDescriptor } from "@xrf/ipc/types/xrf-dialog";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip, Stack } from "@xrf/ui/material";
import { EditorPanelEmpty } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelEmpty";
import { EditorPanelProperty } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelProperty";
import { EditorPanelSection } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelSection";
import { ReactElement, useMemo } from "react";

import { DIALOG_NODE_ID } from "@/applications/dialogs-editor/lib";
import { groupDialogElements, IDialogElementGroup } from "@/applications/dialogs-editor/lib/dialog-elements";
import { DialogViewService } from "@/applications/dialogs-editor/services/dialog-view";
import { EditorPanel } from "@/core/shell/editor/EditorPanel";

import { DialogInspectorSection } from "./DialogInspectorSection";

/**
 * What one node of the open dialog carries.
 */
export function DialogInspectorPanel({
  "data-testid": dataTestId = "dialog-inspector-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: DialogViewService = useInjection(DialogViewService);

  const dialog: Nullable<DialogDescriptor> = viewService.dialog.value;
  const nodeId: Nullable<string> = viewService.inspectedNodeId;
  const isDialogRoot: boolean = nodeId === DIALOG_NODE_ID;

  const phrase: Nullable<DialogPhraseDescriptor> = useMemo(
    () => dialog?.phrases.find((it: DialogPhraseDescriptor) => it.id === nodeId) ?? null,
    [dialog, nodeId]
  );

  // Chosen inside the memo rather than beside it: `?? []` mints a new array every render, so a memo
  // keyed on it would never hold and would regroup on every keystroke elsewhere in the application.
  const groups: Array<IDialogElementGroup> = useMemo(() => {
    const elements: ReadonlyArray<DialogElementDescriptor> = isDialogRoot
      ? (dialog?.elements ?? [])
      : (phrase?.elements ?? []);

    return groupDialogElements(elements);
  }, [dialog, isDialogRoot, phrase]);

  if (!dialog || !nodeId) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Nothing selected"}>
        <EditorPanelEmpty label={"Pick a node on the canvas to see the text and conditions it carries."} />
      </EditorPanel>
    );
  }

  // A node the dialog no longer holds: the canvas keeps its selection across a re-fetch of the same
  // dialog, which may come back without it.
  if (!isDialogRoot && !phrase) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Phrase is gone"}>
        <EditorPanelEmpty label={`This dialog no longer declares '${nodeId}'.`} />
      </EditorPanel>
    );
  }

  const badges: Array<ReactElement> = [
    ...(isDialogRoot && dialog.priority !== null
      ? [<Chip key={"priority"} size={"small"} variant={"outlined"} label={`priority ${dialog.priority}`} />]
      : []),
    // Terminality is worded here where there is room for it, and the two spellings read alike because
    // they behave alike: the engine closes the dialog whether a phrase says `is_final` or simply
    // offers nothing. Nearly four in ten phrases are the second kind.
    ...(phrase && (phrase.isFinal || !phrase.next.length)
      ? [<Chip key={"terminal"} size={"small"} variant={"outlined"} label={phrase.isFinal ? "final" : "ends here"} />]
      : []),
    ...(phrase?.textKey && !phrase.text
      ? [<Chip key={"untranslated"} size={"small"} color={"warning"} variant={"outlined"} label={"untranslated"} />]
      : []),
  ];

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={isDialogRoot ? "Dialog" : "Phrase"}>
      <EditorPanelSection title={"Details"} isFirst>
        <EditorPanelProperty label={"ID"} value={isDialogRoot ? dialog.id : phrase?.id} isMonospace />

        {isDialogRoot ? (
          <>
            <EditorPanelProperty label={"Phrases"} value={dialog.phrases.length} />
            {dialog.language ? <EditorPanelProperty label={"Language"} value={dialog.language} /> : null}
          </>
        ) : (
          <>
            <EditorPanelProperty label={"Text"} value={phrase?.text ?? "No text for this language."} />

            {phrase?.textKey ? <EditorPanelProperty label={"Text key"} value={phrase.textKey} isMonospace /> : null}
          </>
        )}

        {badges.length ? (
          <Stack className={"mt-2.5 flex-wrap gap-y-1.5"} direction={"row"} spacing={0.75}>
            {badges}
          </Stack>
        ) : null}
      </EditorPanelSection>

      {groups.length ? (
        groups.map((group: IDialogElementGroup) => (
          <DialogInspectorSection
            key={group.id}
            title={group.title}
            caption={group.caption}
            elements={group.elements}
          />
        ))
      ) : (
        <EditorPanelEmpty
          label={isDialogRoot ? "This dialog gates nothing." : "This phrase carries no conditions or effects."}
        />
      )}
    </EditorPanel>
  );
}
