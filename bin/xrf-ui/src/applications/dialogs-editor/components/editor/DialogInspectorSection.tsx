import { ReactElement } from "react";

import { DialogElementDescriptor } from "@/core/ipc/types/xrf-dialog";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
