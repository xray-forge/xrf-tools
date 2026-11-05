import { Nullable } from "@xrf/types";
import { EditorPopoverRow, IEditorPopoverRowControlProps } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FilePickerInput } from "@xrf/ui/form/file-picker";
import { ReactElement, useId } from "react";

import { IPathField } from "@/core/ui/form/use-path-field";

interface IPathPanelRowProps extends BaseComponentProps {
  label: string;
  /** What the path is for, read on hovering the label and by a screen reader. */
  description?: string;
  isDisabled?: boolean;
  placeholder?: string;
  field: IPathField;
}

/**
 * A remembered, validated path among a panel's rows: the label over the field, as a path is too long to share a row
 * with its label in a narrow panel, the field as low as the rows around it, and what the validation found under it.
 *
 * Unlike `PathFormRow` it joins no form, since a panel has none: the action that reads the path commits it.
 */
export function PathPanelRow({
  "data-testid": dataTestId = "path-panel-row",
  id,
  className,
  label,
  description,
  isDisabled = false,
  placeholder,
  field,
}: IPathPanelRowProps): ReactElement {
  const errorId: string = useId();
  const error: Nullable<string> = field.error;

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <EditorPopoverRow label={label} description={description} isDisabled={isDisabled} isStacked>
        {({ id: controlId, "aria-describedby": descriptionId }: IEditorPopoverRowControlProps) => (
          <FilePickerInput
            id={controlId}
            aria-describedby={[descriptionId, error ? errorId : null].filter(Boolean).join(" ") || undefined}
            placeholder={placeholder}
            value={field.value}
            isDisabled={isDisabled}
            isInvalid={Boolean(error)}
            isCompact
            recents={field.recents}
            onSelect={field.select}
            onChange={field.setValue}
            onClear={field.clear}
          />
        )}
      </EditorPopoverRow>

      {error ? (
        <p id={errorId} className={"mb-1 text-xs text-error"}>
          {error}
        </p>
      ) : null}
    </div>
  );
}
