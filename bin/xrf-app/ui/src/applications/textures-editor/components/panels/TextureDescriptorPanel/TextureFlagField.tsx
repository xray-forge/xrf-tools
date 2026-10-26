import { Checkbox, FormControlLabel } from "@mui/material";
import { ChangeEvent, ReactElement, useCallback } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

interface ITextureFlagFieldProps extends BaseComponentProps {
  label: string;
  isChecked: boolean;
  onChange: (isChecked: boolean) => void;
}

/**
 * One bit of the flag word, under the SDK's own identifier for it.
 *
 * Labelled `flHasAlpha` rather than "Has alpha", because the identifier is what an author of a `.thm` recognises and
 * what every other tool and every piece of documentation calls it.
 */
export function TextureFlagField({
  "data-testid": dataTestId = "texture-flag-field",
  id,
  className,
  label,
  isChecked,
  onChange,
}: ITextureFlagFieldProps): ReactElement {
  const handleChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => onChange(event.target.checked),
    [onChange]
  );

  return (
    <FormControlLabel
      data-testid={dataTestId}
      id={id}
      className={className}
      control={<Checkbox size={"small"} checked={isChecked} onChange={handleChange} />}
      label={label}
      slotProps={{ typography: { variant: "body2" } }}
    />
  );
}
