// Different environments use different variables
import {orderableDocumentListDeskItem} from "@sanity/orderable-document-list";

const projectId =
    import.meta.env.PUBLIC_SANITY_STUDIO_PROJECT_ID! ||
    import.meta.env.PUBLIC_SANITY_PROJECT_ID!;
const dataset =
    import.meta.env.PUBLIC_SANITY_STUDIO_DATASET! ||
    import.meta.env.PUBLIC_SANITY_DATASET!;

// Feel free to remove this check if you don't need it
if (!projectId || !dataset) {
    throw new Error(
        `Missing environment variable(s). Check if named correctly in .env file.\n\nShould be:\nPUBLIC_SANITY_STUDIO_PROJECT_ID=${projectId}\nPUBLIC_SANITY_STUDIO_DATASET=${dataset}\n\nAvailable environment variables:\n${JSON.stringify(
            import.meta.env,
            null,
            2
        )}`
    );
}

import {defineConfig} from "sanity";
import {deskTool} from "sanity/desk";
import {visionTool} from "@sanity/vision";
import {schemaTypes} from "./schema";

export default defineConfig({
    name: "juletips",
    title: "Juletips",
    projectId,
    dataset,
    plugins: [deskTool({
        structure: (S, context) => {
            return S.list()
                .title('Content')
                .items([
                    orderableDocumentListDeskItem({
                        type: 'post',
                        S,
                        context
                    }),
                ])
        },
    }), visionTool()],
    schema: {
        types: schemaTypes,
    },
});
