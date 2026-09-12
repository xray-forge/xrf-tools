import { Box, MenuItem, TextField, Typography } from "@mui/material";
import { ReactElement } from "react";

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
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", alignItems: "center", gap: 1.5, flexWrap: "wrap" }}
    >
      <TextField
        select
        size={"small"}
        label={"Reference"}
        value={languages.includes(reference) ? reference : ""}
        sx={{ minWidth: 180 }}
        onChange={(event) => onReferenceChange(event.target.value)}
      >
        {languages.map((it: string) => (
          <MenuItem key={it} value={it}>
            {describe(it, encodings)}
          </MenuItem>
        ))}
      </TextField>

      <Typography aria-hidden={true} variant={"body2"} sx={{ color: "text.secondary" }}>
        →
      </Typography>

      <TextField
        select
        size={"small"}
        label={"Target"}
        value={languages.includes(target) ? target : ""}
        sx={{ minWidth: 180 }}
        onChange={(event) => onTargetChange(event.target.value)}
      >
        {languages.map((it: string) => (
          <MenuItem key={it} value={it}>
            {describe(it, encodings)}
          </MenuItem>
        ))}
      </TextField>
    </Box>
  );
}
