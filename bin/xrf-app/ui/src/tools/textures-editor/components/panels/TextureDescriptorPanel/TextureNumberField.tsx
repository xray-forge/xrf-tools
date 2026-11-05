import { Nullable } from "@xrf/types";
import { EditorPopoverTextRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback } from "react";

interface ITextureNumberFieldProps extends BaseComponentProps {
  label: string;
  /**
   * Null where the descriptor holds a value JSON cannot carry, which for an `f32` means non-finite.
   */
  value: Nullable<number>;
  /** What the field means, read on hovering its label. */
  description?: string;
  isReadOnly?: boolean;
  onChange?: (value: number) => void;
}

/**
 * One numeric descriptor field.
 */
export function TextureNumberField({
  "data-testid": dataTestId = "texture-number-field",
  id,
  className,
  label,
  value,
  description,
  isReadOnly = false,
  onChange,
}: ITextureNumberFieldProps): ReactElement {
  const handleChange = useCallback(
    (typed: string) => {
      const parsed: number = Number(typed);

      // A field mid-edit can read as empty or as a lone minus sign, which parse to NaN and would otherwise be written
      // into the draft and then into the file.
      if (onChange && Number.isFinite(parsed)) {
        onChange(parsed);
      }
    },
    [onChange]
  );

  return (
    <EditorPopoverTextRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={description}
      type={"number"}
      value={value ?? ""}
      isReadOnly={isReadOnly}
      onChange={onChange ? handleChange : undefined}
    />
  );
}
