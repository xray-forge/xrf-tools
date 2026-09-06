import { Alert, Button, Stack, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { TextureGlossField } from "@/applications/textures-editor/components/panels/TextureBumpPanel/TextureGlossField";
import { toBumpReference, toCompanionReference } from "@/applications/textures-editor/lib/texture-bump-target";
import { TextureBumpService } from "@/applications/textures-editor/services/bump";
import { TextureEditorService } from "@/applications/textures-editor/services/editor";
import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { EApplicationId } from "@/core/routing/application";
import { EditorPanel, EditorPanelEmpty, EditorPanelRow, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** A gloss above the SDK's warning threshold, for a surface with no mask to read one from. */
const DEFAULT_GLOSS: number = 0.5;

/** Images the generator can read a plane out of, which is whatever `image` decodes. */
const IMAGE_FILTERS = [{ extensions: ["png", "tga", "bmp", "jpg", "jpeg"], name: "Image" }];

/**
 * Build the `_bump` and `_bump#` pair this texture binds, from a height map.
 *
 * A port of the SDK's own generator, so the inputs are its inputs: the relief is required and everything else refines
 * it. There is no destination field - the pair belongs to the texture that is open, and its two names are derived, not
 * chosen.
 */
export function TextureBumpPanel({
  "data-testid": dataTestId = "texture-bump-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const editorService: TextureEditorService = useInjection(TextureEditorService);
  const bumpService: TextureBumpService = useInjection(TextureBumpService);

  const [glossConstant, setGlossConstant] = useState<number>(DEFAULT_GLOSS);

  const description: Nullable<TextureDescription> = selectionService.selected.value;
  const isRunning: boolean = bumpService.generate.isRunning;

  const height: IPathField = usePathField({
    application: EApplicationId.TEXTURES_EDITOR,
    filters: IMAGE_FILTERS,
    id: "bump-height",
    isDisabled: isRunning,
    title: "Select the height map",
  });

  const gloss: IPathField = usePathField({
    application: EApplicationId.TEXTURES_EDITOR,
    filters: IMAGE_FILTERS,
    id: "bump-gloss",
    isDisabled: isRunning,
    title: "Select a gloss mask",
  });

  const normalMap: IPathField = usePathField({
    application: EApplicationId.TEXTURES_EDITOR,
    filters: IMAGE_FILTERS,
    id: "bump-normal-map",
    isDisabled: isRunning,
    title: "Select an external normal map",
  });

  const onGenerate = useCallback(
    () =>
      void bumpService.run({
        gloss: gloss.value,
        glossConstant,
        height: height.value ?? "",
        normalMap: normalMap.value,
        // The descriptor's own, so generating twice at different depths is a deliberate act rather than a surprise.
        virtualHeight: editorService.draft?.virtualHeight ?? 0.05,
      }),
    [bumpService, editorService.draft?.virtualHeight, gloss.value, glossConstant, height.value, normalMap.value]
  );

  if (!description) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Bump"}>
        <EditorPanelEmpty label={"No texture selected. The generator for its bump pair shows here."} />
      </EditorPanel>
    );
  }

  const bumpReference: string = toBumpReference(description);

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Bump"}>
      {bumpService.destination === null ? (
        <Alert severity={"info"} sx={{ mb: 1 }}>
          This texture is served out of an archive, so there is nowhere beside it to write a pair.
        </Alert>
      ) : null}

      <EditorPanelSection title={"Writes"} caption={"Derived from the texture, not chosen"} isFirst>
        <EditorPanelRow label={"Bump"} value={bumpReference} />
        <EditorPanelRow label={"Companion"} value={toCompanionReference(bumpReference)} />
      </EditorPanelSection>

      <EditorPanelSection title={"Sources"} caption={"The relief is required; everything else refines it"}>
        <PathFormRow
          label={"Height map"}
          description={"Averaged across its colour channels, as the SDK does"}
          isDisabled={isRunning}
          field={height}
        />

        <PathFormRow
          label={"Gloss mask"}
          description={"Optional. Without one, the level below is used everywhere"}
          isDisabled={isRunning}
          field={gloss}
        />

        {gloss.value ? null : (
          <TextureGlossField value={glossConstant} isDisabled={isRunning} onChange={setGlossConstant} />
        )}

        <PathFormRow
          label={"Normal map"}
          description={"Optional. Replaces the normals the height implies, and must match its size"}
          isDisabled={isRunning}
          field={normalMap}
        />
      </EditorPanelSection>

      <EditorPanelSection title={"Generate"}>
        <Stack direction={"row"} spacing={1}>
          <Button
            data-testid={"texture-bump-generate"}
            size={"small"}
            variant={"contained"}
            disabled={!bumpService.canGenerate || !height.isValid}
            onClick={onGenerate}
          >
            Generate pair
          </Button>

          {isRunning ? (
            <Button data-testid={"texture-bump-cancel"} size={"small"} onClick={bumpService.generate.cancel}>
              Stop
            </Button>
          ) : null}
        </Stack>

        <Typography variant={"caption"} color={"text.secondary"} sx={{ display: "block", mt: 1 }}>
          Both halves are written at once, and the descriptor is pointed at them. Saving is still a separate act.
        </Typography>

        {bumpService.generate.error ? (
          <Alert severity={"error"} sx={{ mt: 1 }}>
            {bumpService.generate.error}
          </Alert>
        ) : null}
      </EditorPanelSection>
    </EditorPanel>
  );
}
