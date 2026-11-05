import { EditorPopoverSwitchRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

interface ITextureFlagFieldProps extends BaseComponentProps {
  label: string;
  isChecked: boolean;
  onChange: (isChecked: boolean) => void;
}

/**
 * One bit of the flag word, under the SDK's own identifier for it.
 *
 * Labelled `flHasAlpha` rather than "Has alpha", because the identifier is what an author of a `.thm` recognises and
 * what every other tool and every piece of documentation calls it. The box leads, so the identifier takes the row's
 * whole width rather than a label column too narrow for the longer ones.
 */
export function TextureFlagField({
  "data-testid": dataTestId = "texture-flag-field",
  id,
  className,
  label,
  isChecked,
  onChange,
}: ITextureFlagFieldProps): ReactElement {
  return (
    <EditorPopoverSwitchRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      isChecked={isChecked}
      onChange={onChange}
    />
  );
}
