import {orderableDocumentListDeskItem} from '@sanity/orderable-document-list'
// @ts-ignore
import juleMathias from './static/julemathias.jpg'

// Different environments use different variables
const projectId = process.env.SANITY_STUDIO_PROJECT_ID! || process.env.SANITY_PROJECT_ID!
const dataset = process.env.SANITY_STUDIO_DATASET! || process.env.SANITY_DATASET!

// Feel free to remove this check if you don't need it
if (!projectId || !dataset) {
  throw new Error(
    `Missing environment variable(s). Check if named correctly in .env file.\n\nShould be:\nPUBLIC_SANITY_STUDIO_PROJECT_ID=${projectId}\nPUBLIC_SANITY_STUDIO_DATASET=${dataset}\n\nAvailable environment variables:\n${JSON.stringify(
      process.env,
      null,
      2,
    )}`,
  )
}

import {defineConfig} from 'sanity'
import {visionTool} from '@sanity/vision'
import {schemaTypes} from './schema'
import {structureTool} from 'sanity/structure'

export default defineConfig({
  name: 'juletips',
  title: 'Juletips',
  icon: () => (
    <img
      alt={'studio'}
      src={juleMathias}
      style={{objectFit: 'cover', height: '100%', width: '100%'}}
    />
  ),
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
          ])
      },
    }),
    visionTool(),
  ],
  tools: (prev) => prev.filter((tool) => tool.name !== 'schedules'),
  schema: {
    types: schemaTypes,
  },
})
