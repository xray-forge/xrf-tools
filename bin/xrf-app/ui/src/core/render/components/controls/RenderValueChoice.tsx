import { Nullable } from "@xrf/types";
import { EditorPopoverRow, IEditorPopoverRowControlProps } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { ToggleButton, ToggleButtonGroup } from "@xrf/ui/material/toggle-button";
import { ReactElement } from "react";

export interface IRenderValueChoiceProps<T extends string> extends BaseComponentProps {
  label: string;
  /** What it does, read on hovering its label. */
  description?: string;
  options: ReadonlyArray<IChoiceFormRowOption<T>>;
  value: T;
  /** Whether the values stack one under another, under the label, for names too long to share a row. */
  isStacked?: boolean;
  isDisabled?: boolean;
  onChange: (value: T) => void;
}

/**
 * One of a few values a preview is driven by, in one row of its popover: its name and the values beside it, one kept
 * selected.
 */
export function RenderValueChoice<T extends string>({
  "data-testid": dataTestId = "render-value-choice",
  id,
  className,
  label,
  description,
  options,
  value,
  isStacked = false,
  isDisabled = false,
  onChange,
}: IRenderValueChoiceProps<T>): ReactElement {
  return (
    <EditorPopoverRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={description}
      isDisabled={isDisabled}
      isStacked={isStacked}
    >
      {(control: IEditorPopoverRowControlProps) => (
        <ToggleButtonGroup
          {...control}
          exclusive={true}
          fullWidth={true}
          orientation={isStacked ? "vertical" : "horizontal"}
          color={"primary"}
          size={"small"}
          value={value}
          onChange={(_, next: Nullable<T>) => {
            if (next !== null) {
              onChange(next);
            }
          }}
        >
          {options.map((option: IChoiceFormRowOption<T>) => (
            <ToggleButton
              key={option.value}
              aria-label={option["aria-label"]}
              className={"py-0.5 text-xs"}
              value={option.value}
              disabled={isDisabled || option.isDisabled}
            >
              {option.label}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      )}
    </EditorPopoverRow>
  );
}
