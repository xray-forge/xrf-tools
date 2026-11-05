import { EditorPopoverTextRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

interface ITextureTextFieldProps extends BaseComponentProps {
  label: string;
  value: string;
  /** What the field means, read on hovering its label. */
  description?: string;
  onChange: (value: string) => void;
}

/**
 * One free-text descriptor field, such as an engine reference the descriptor names.
 */
export function TextureTextField({
  "data-testid": dataTestId = "texture-text-field",
  id,
  className,
  label,
  value,
  description,
  onChange,
}: ITextureTextFieldProps): ReactElement {
  return (
    <EditorPopoverTextRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={description}
      value={value}
      onChange={onChange}
    />
  );
}
