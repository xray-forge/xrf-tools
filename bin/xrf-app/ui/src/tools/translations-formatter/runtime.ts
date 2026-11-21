import { IContainerDefinition } from "@/lib/container/container-definition";
import { TranslationsFormatterService } from "@/tools/translations-formatter/services/translations-formatter.service";

export { TranslationsFormatterTool as Component } from "./TranslationsFormatterTool";

export const container: IContainerDefinition = { bindings: [TranslationsFormatterService] };
