// Different environments use different variables
import { orderableDocumentListDeskItem } from '@sanity/orderable-document-list';

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
      2,
    )}`,
  );
}

import { defineConfig } from 'sanity';
import { visionTool } from '@sanity/vision';
import { schemaTypes } from './schema';
import { vercelDeployTool } from 'sanity-plugin-vercel-deploy';
import { structureTool } from 'sanity/structure';

export default defineConfig({
  name: 'juletips',
  title: 'Juletips',
  projectId,
  dataset,
  plugins: [
    structureTool({
      structure: (S, context) => {
        return S.list()
          .title('Content')
          .items([
            orderableDocumentListDeskItem({
              type: 'wish',
              S,
              context,
            }),
          ]);
      },
    }),
    visionTool(),
    vercelDeployTool(),
  ],
  schema: {
    types: schemaTypes,
  },
});
