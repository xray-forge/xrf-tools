import { MenuItem, Select } from "@mui/material";
import { ReactElement } from "react";

import { ETranslationLanguage } from "@/core/ipc/types/xrf-translation";
import { TRANSLATION_LANGUAGES, TRANSLATION_LANGUAGES_WITH_ALL } from "@/core/translations/translations.config";
import { FormRow } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ITranslationLanguageFieldProps extends BaseComponentProps {
  description: string;
  value: ETranslationLanguage;
  /** Whether the command accepts every language in one run. */
  isAllAllowed?: boolean;
  isDisabled?: boolean;
  onChange: (language: ETranslationLanguage) => void;
}

/** A controlled language selector with its label and command-specific description. */
export function TranslationLanguageField({
  "data-testid": dataTestId = "translation-language-field",
  id,
  className,
  description,
  value,
  isAllAllowed = false,
  isDisabled = false,
  onChange,
}: ITranslationLanguageFieldProps): ReactElement {
  const languages: ReadonlyArray<ETranslationLanguage> = isAllAllowed
    ? TRANSLATION_LANGUAGES_WITH_ALL
    : TRANSLATION_LANGUAGES;

  return (
    <FormRow label={"Language"} description={description} controlId={id} isInline>
      {({ id: controlId, "aria-labelledby": labelId, "aria-describedby": describedBy }) => (
        <Select
          data-testid={dataTestId}
          id={controlId}
          className={className}
          labelId={labelId}
          aria-describedby={describedBy}
          size={"small"}
          value={value}
          disabled={isDisabled}
          onChange={(event) => onChange(event.target.value)}
        >
          {languages.map((language) => (
            <MenuItem key={language} value={language}>
              {language}
            </MenuItem>
          ))}
        </Select>
      )}
    </FormRow>
  );
}
