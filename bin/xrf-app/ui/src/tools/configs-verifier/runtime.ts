import { IContainerDefinition } from "@/lib/container/container-definition";
import { ConfigsVerifierService } from "@/tools/configs-verifier/services/configs-verifier.service";

export { ConfigsVerifierTool as Component } from "./ConfigsVerifierTool";

export const container: IContainerDefinition = { bindings: [ConfigsVerifierService] };
