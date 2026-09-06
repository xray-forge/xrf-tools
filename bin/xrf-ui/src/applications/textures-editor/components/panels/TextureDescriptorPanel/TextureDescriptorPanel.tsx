import { Alert, Box, Button, Stack } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect } from "react";

import { hasTextureFlag, withTextureFlag } from "@/applications/textures-editor/lib/texture-descriptor-flags";
import { TextureEditorService } from "@/applications/textures-editor/services/editor";
import {
  TextureDescription,
  TextureDescriptorForm,
  TextureFlagEntry,
  TextureVocabulary,
} from "@/core/bindings/types/xrf-app";
import { EditorPanel, EditorPanelEmpty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { TextureFlagField } from "./TextureFlagField";
import { TextureNumberField } from "./TextureNumberField";
import { TextureSelectField } from "./TextureSelectField";
import { TextureTextField } from "./TextureTextField";

/** Flags that belong with the fields a person changes to change how a surface looks. */
const PRIMARY_FLAG_LABELS: ReadonlyArray<string> = ["flDiffuseDetail", "flBumpDetail", "flHasAlpha", "flBinaryAlpha"];

/**
 * The `.thm` as a form: what the engine reads above, what only the converter reads behind a disclosure.
 */
export function TextureDescriptorPanel({
  "data-testid": dataTestId = "texture-descriptor-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const editorService: TextureEditorService = useInjection(TextureEditorService);

  const description: Nullable<TextureDescription> = selectionService.selected.value;
  const vocabulary: Nullable<TextureVocabulary> = editorService.vocabulary.value;
  const draft: Nullable<TextureDescriptorForm> = editorService.draft;

  const onEdit = useCallback((patch: Partial<TextureDescriptorForm>) => editorService.edit(patch), [editorService]);

  const onFlag = useCallback(
    (bit: number, isSet: boolean) =>
      editorService.edit({ flags: withTextureFlag(editorService.draft?.flags ?? 0, bit, isSet) }),
    [editorService]
  );

  useEffect(() => editorService.bind(description), [editorService, description]);

  if (!description || !draft || !vocabulary) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Descriptor"}>
        <EditorPanelEmpty
          label={
            description
              ? "Reading the descriptor vocabulary."
              : "No texture selected. Its .thm fields show here, and can be edited."
          }
        />
      </EditorPanel>
    );
  }

  const isAuthoring: boolean = description.form === null;
  const primaryFlags: Array<TextureFlagEntry> = vocabulary.flags.filter((flag: TextureFlagEntry) =>
    PRIMARY_FLAG_LABELS.includes(flag.label)
  );

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Descriptor"}>
      {description.targets === null ? (
        <Alert severity={"info"} sx={{ mb: 1 }}>
          This texture is served out of an archive, so there is no file to write. Fields can be read, not saved.
        </Alert>
      ) : null}

      {isAuthoring ? (
        <Alert severity={"info"} sx={{ mb: 1 }}>
          No .thm sits beside this texture. Saving writes one, starting from the defaults the SDK begins a new
          descriptor at.
        </Alert>
      ) : null}

      <EditorPanelSection title={"Primary"} caption={"What the engine reads at load time"} isFirst>
        <TextureSelectField
          data-testid={"texture-field-texture-type"}
          label={"Texture type"}
          entries={vocabulary.textureTypes}
          value={draft.textureType}
          helperText={"A type outside Image makes LoadTHM skip the descriptor whole"}
          onChange={(textureType: number) => onEdit({ textureType })}
        />

        <TextureSelectField
          data-testid={"texture-field-bump-mode"}
          label={"Bump mode"}
          entries={vocabulary.bumpModes}
          value={draft.bumpMode}
          onChange={(bumpMode: number) => onEdit({ bumpMode })}
        />

        <TextureTextField
          data-testid={"texture-field-bump-name"}
          label={"Bump texture"}
          value={draft.bumpName}
          helperText={"Taken verbatim; there is no _bump naming convention behind it"}
          onChange={(bumpName: string) => onEdit({ bumpName })}
        />

        <TextureTextField
          data-testid={"texture-field-detail-name"}
          label={"Detail texture"}
          value={draft.detailName}
          onChange={(detailName: string) => onEdit({ detailName })}
        />

        <TextureNumberField
          data-testid={"texture-field-detail-scale"}
          label={"Detail scale"}
          value={draft.detailScale}
          onChange={(detailScale: number) => onEdit({ detailScale })}
        />

        <Box sx={{ display: "flex", flexDirection: "column", mt: 1 }}>
          {primaryFlags.map((flag: TextureFlagEntry) => (
            <TextureFlagField
              key={flag.bit}
              data-testid={`texture-flag-${flag.label}`}
              label={flag.label}
              isChecked={hasTextureFlag(draft.flags, flag.bit)}
              onChange={(isSet: boolean) => onFlag(flag.bit, isSet)}
            />
          ))}
        </Box>
      </EditorPanelSection>

      <EditorPanelSection
        title={"Authoring metadata"}
        caption={"Read by the converter that built the file, and by nothing at runtime"}
      >
        <TextureSelectField
          data-testid={"texture-field-format"}
          label={"Format"}
          entries={vocabulary.formats}
          value={draft.format}
          onChange={(format: number) => onEdit({ format })}
        />

        <TextureSelectField
          data-testid={"texture-field-mip-filter"}
          label={"Mip filter"}
          entries={vocabulary.mipFilters}
          value={draft.mipFilter}
          onChange={(mipFilter: number) => onEdit({ mipFilter })}
        />

        <TextureSelectField
          data-testid={"texture-field-material"}
          label={"Material"}
          entries={vocabulary.materials}
          value={draft.material}
          helperText={"Reaches the renderer through the descriptor manager, unlike the rest of this group"}
          onChange={(material: number) => onEdit({ material })}
        />

        <TextureNumberField
          data-testid={"texture-field-material-weight"}
          label={"Material weight"}
          value={draft.materialWeight}
          onChange={(materialWeight: number) => onEdit({ materialWeight })}
        />

        <TextureNumberField
          data-testid={"texture-field-virtual-height"}
          label={"Virtual height"}
          value={draft.virtualHeight}
          helperText={"Not read at runtime; the bump generator reads it"}
          onChange={(virtualHeight: number) => onEdit({ virtualHeight })}
        />

        <TextureTextField
          data-testid={"texture-field-ext-normal-map"}
          label={"External normal map"}
          value={draft.extNormalMapName}
          onChange={(extNormalMapName: string) => onEdit({ extNormalMapName })}
        />

        <TextureNumberField
          data-testid={"texture-field-fade-delay"}
          label={"Fade delay"}
          value={draft.fadeDelay}
          onChange={(fadeDelay: number) => onEdit({ fadeDelay })}
        />

        <TextureNumberField
          data-testid={"texture-field-width"}
          label={"Width"}
          value={draft.width}
          helperText={"From the DDS header; the descriptor's own copy is refreshed on save"}
          isReadOnly
        />

        <TextureNumberField data-testid={"texture-field-height"} label={"Height"} value={draft.height} isReadOnly />

        <Box sx={{ display: "flex", flexDirection: "column", mt: 1 }}>
          {vocabulary.flags
            .filter((flag: TextureFlagEntry) => !PRIMARY_FLAG_LABELS.includes(flag.label))
            .map((flag: TextureFlagEntry) => (
              <TextureFlagField
                key={flag.bit}
                data-testid={`texture-flag-${flag.label}`}
                label={flag.label}
                isChecked={hasTextureFlag(draft.flags, flag.bit)}
                onChange={(isSet: boolean) => onFlag(flag.bit, isSet)}
              />
            ))}
        </Box>
      </EditorPanelSection>

      <EditorPanelSection title={"Changes"}>
        <Stack direction={"row"} spacing={1}>
          <Button
            data-testid={"texture-descriptor-save"}
            size={"small"}
            variant={"contained"}
            disabled={!editorService.canSave}
            onClick={() => void editorService.commit()}
          >
            Save
          </Button>

          <Button
            data-testid={"texture-descriptor-discard"}
            size={"small"}
            variant={"outlined"}
            disabled={!editorService.isDirty || editorService.save.isRunning}
            onClick={editorService.discard}
          >
            Discard
          </Button>
        </Stack>

        {editorService.save.error ? (
          <Alert severity={"error"} sx={{ mt: 1 }}>
            {editorService.save.error}
          </Alert>
        ) : null}
      </EditorPanelSection>
    </EditorPanel>
  );
}
