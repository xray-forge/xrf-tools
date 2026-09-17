import { MenuItem, TextField, Typography } from "@mui/material";
import { ReactElement } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

function describe(language: string, encodings: Record<string, string>): string {
  return encodings[language] ? `${language} · ${encodings[language]}` : language;
}

export interface ITranslationsLanguageBarProps extends BaseComponentProps {
  languages: ReadonlyArray<string>;
  encodings: Record<string, string>;
  reference: string;
  target: string;
  onReferenceChange: (language: string) => void;
  onTargetChange: (language: string) => void;
}

/**
 * Which language is being translated, and which one from.
 */
export function TranslationsLanguageBar({
  "data-testid": dataTestId = "translations-language-bar",
  id,
  className,
  languages,
  encodings,
  reference,
  target,
  onReferenceChange,
  onTargetChange,
}: ITranslationsLanguageBarProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-wrap items-center gap-3", className)}>
      <TextField
        className={"min-w-45"}
        select={true}
        size={"small"}
        label={"Reference"}
        value={languages.includes(reference) ? reference : ""}
        onChange={(event) => onReferenceChange(event.target.value)}
      >
        {languages.map((it: string) => (
          <MenuItem key={it} value={it}>
            {describe(it, encodings)}
          </MenuItem>
        ))}
      </TextField>

      <Typography aria-hidden={true} className={"text-text-secondary"} variant={"body2"}>
        →
      </Typography>

      <TextField
        className={"min-w-45"}
        select={true}
        size={"small"}
        label={"Target"}
        value={languages.includes(target) ? target : ""}
        onChange={(event) => onTargetChange(event.target.value)}
      >
        {languages.map((it: string) => (
          <MenuItem key={it} value={it}>
            {describe(it, encodings)}
          </MenuItem>
        ))}
      </TextField>
    </div>
  );
}
