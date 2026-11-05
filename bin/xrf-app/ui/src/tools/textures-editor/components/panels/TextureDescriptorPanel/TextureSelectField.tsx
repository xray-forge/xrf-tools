import { TextureVocabularyEntry } from "@xrf/ipc/types/xrf-app-textures";
import { EditorPopoverSelectRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { MenuItem } from "@xrf/ui/material/menu";
import { ReactElement } from "react";

import { toVocabularyOptions } from "@/tools/textures-editor/lib/texture-vocabulary";

interface ITextureSelectFieldProps extends BaseComponentProps {
  label: string;
  /** The named values the SDK offers for this field. */
  entries: ReadonlyArray<TextureVocabularyEntry>;
  value: number;
  /** What the field means, read on hovering its label. */
  description?: string;
  onChange: (value: number) => void;
}

/**
 * One enumerated descriptor field, offering the values the SDK names plus whatever this file happens to hold.
 */
export function TextureSelectField({
  "data-testid": dataTestId = "texture-select-field",
  id,
  className,
  label,
  entries,
  value,
  description,
  onChange,
}: ITextureSelectFieldProps): ReactElement {
  return (
    <EditorPopoverSelectRow<string>
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={description}
      value={String(value)}
      onChange={(selected: string) => onChange(Number(selected))}
    >
      {toVocabularyOptions(entries, value).map((entry: TextureVocabularyEntry) => (
        <MenuItem key={entry.value} value={String(entry.value)}>
          {entry.label}
        </MenuItem>
      ))}
    </EditorPopoverSelectRow>
  );
}
