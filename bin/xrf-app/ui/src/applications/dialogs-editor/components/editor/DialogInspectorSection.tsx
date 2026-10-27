import { DialogElementDescriptor } from "@xrf/ipc/types/xrf-dialog";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EditorPanelProperty } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelProperty";
import { EditorPanelSection } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelSection";
import { ReactElement } from "react";

export interface IDialogInspectorSectionProps extends BaseComponentProps {
  title: string;
  caption: string;
  elements: ReadonlyArray<DialogElementDescriptor>;
}

/**
 * One titled group of a node's elements.
 */
export function DialogInspectorSection({
  "data-testid": dataTestId,
  id,
  className,
  title,
  caption,
  elements,
}: IDialogInspectorSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={title} caption={caption}>
      {elements.map((element: DialogElementDescriptor, index: number) => (
        <EditorPanelProperty key={`${element.name}-${index}`} label={element.name} value={element.value} isMonospace />
      ))}
    </EditorPanelSection>
  );
}
