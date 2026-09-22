import { default as AccountTreeIcon } from "@mui/icons-material/AccountTree";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useMemo } from "react";

import { isAlphaRenderSurface } from "@/core/render/lib/surface/render-surface";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { IEditorPanel, useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { VisualPreviewEmpty, VisualPreviewToolbar } from "@/core/visuals/components/preview";
import { VisualPreviewViewport } from "@/core/visuals/components/preview/VisualPreviewViewport";
import { IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { countVisualTriangles, IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IVisualPreviewLayoutProps extends BaseComponentProps {
  /** Shown in the toolbar beside the view toggles, usually where the model came from. */
  subtitle?: ReactNode;
  /** What the open visual is called. Its presence is what draws the file header over the viewport. */
  name?: Nullable<string>;
  /** Published as a left panel when given. Opening a single visual has nothing to browse. */
  tree?: ReactNode;
  /** Data panels the owning application contributes to the right stripe. */
  panels?: Array<IEditorPanel>;
  /** Drawn under the viewport, at whatever height it asks for. */
  footer?: ReactNode;
  /** Whether a model is on its way, reported over the viewport rather than by replacing the screen. */
  isLoading?: boolean;
  /** Why the last open failed, shown in place of a model rather than dismissing the session. */
  error?: string;
  /** Reads the failed open's source again. Absent while an application cannot repeat its last attempt. */
  onRetry?: () => void;
  /** Reopens the picker. Absent while an application has no way to choose a different visual. */
  onBack?: () => void;
  /** Promotes a single-model session to a browsed one. Absent while already browsing. */
  onBrowse?: () => void;
  /** Ends the selection without ending the session, which is what puts the close action in the file header. */
  onDeselect?: Nullable<() => void>;
}

/**
 * The shared preview chrome: toolbar, viewport, panel stripe and animation bar.
 */
export function VisualPreviewLayout({
  "data-testid": dataTestId = "visual-preview-layout",
  id = "visual-preview-layout",
  className,
  subtitle,
  name = null,
  tree,
  panels,
  footer,
  isLoading = false,
  error,
  onRetry,
  onBack,
  onBrowse,
  onDeselect = null,
}: IVisualPreviewLayoutProps): ReactElement {
  const viewService: VisualViewService = useInjection(VisualViewService);
  const source: IVisualRenderSource = useInjection(VISUAL_RENDER_SOURCE);

  const model: Nullable<IVisualModelViews> = source.model;
  const triangleCount: number = model ? countVisualTriangles(model, viewService.detail) : 0;
  const hasDetailLevels: boolean = (model?.levelCount ?? 1) > 1;
  const hasSkeleton: boolean = Boolean(model?.skeleton);
  // A dummy pair counts: it is uploaded and shaded, and comparing it flat is how a modder sees that it adds nothing.
  const hasBump: boolean = Boolean(model && source.bumps.size > 0);
  const hasAlpha: boolean = Boolean(model?.submeshes.some((submesh) => isAlphaRenderSurface(submesh.surface)));

  useEditorPanels(() => {
    const stripe: Array<IEditorPanel> = panels ? [...panels] : [];

    return tree
      ? [
          {
            icon: <AccountTreeIcon />,
            id: "project",
            isOpenByDefault: true,
            label: "Project",
            render: () => tree,
            side: "left",
          },
          ...stripe,
        ]
      : stripe;
  }, [tree, panels]);

  const status: Array<string> = useMemo(() => {
    if (isLoading) {
      return ["Loading visual"];
    }

    return model
      ? [`${model.submeshes.length} submeshes`, `${model.vertexCount} vertices`, `${triangleCount} triangles`]
      : ["No visual open"];
  }, [isLoading, model, triangleCount]);

  useEditorStatus(status);

  return (
    <EditorLayout
      toolbar={
        <VisualPreviewToolbar
          hasDetailLevels={hasDetailLevels}
          hasSkeleton={hasSkeleton}
          hasBump={hasBump}
          hasAlpha={hasAlpha}
          subtitle={subtitle}
          options={viewService.options}
          lighting={viewService.lighting}
          detail={viewService.detail}
          onChangeOptions={viewService.setOptions}
          onChangeLighting={viewService.setLighting}
          onChangeDetail={viewService.setDetail}
          onBack={onBack}
          onBrowse={onBrowse}
        />
      }
      footer={footer}
    >
      <div className={"flex min-h-0 min-w-0 grow flex-col"}>
        {name ? (
          <EditorFileHeader
            data-testid={"visual-file-header"}
            name={name}
            closeLabel={"Close visual"}
            closeDescription={"Clear the selection and close this visual"}
            onClose={onDeselect ?? undefined}
          />
        ) : null}

        <div
          data-testid={dataTestId}
          id={id}
          className={cn("relative flex min-h-0 min-w-0 flex-1 overflow-hidden", className)}
        >
          <VisualPreviewViewport />

          {!model && !isLoading ? <VisualPreviewEmpty error={error} onRetry={onRetry} /> : null}

          {isLoading ? (
            <div className={"pointer-events-none absolute inset-0 flex items-center justify-center"}>
              <DelayedProgress isOnViewport={true} label={"Loading visual…"} />
            </div>
          ) : null}
        </div>
      </div>
    </EditorLayout>
  );
}
