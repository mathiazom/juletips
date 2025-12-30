import { type ClientConfig, createClient } from '@sanity/client';

const projectId =
  import.meta.env.SANITY_STUDIO_PROJECT_ID!;
const dataset =
  import.meta.env.SANITY_STUDIO_DATASET!;

const config: ClientConfig = {
  projectId,
  dataset,
  useCdn: true, // set to `false` to bypass the edge cache
  apiVersion: '2025-02-06', // use current date (YYYY-MM-DD) to target the latest API version. Note: this should always be hard coded. Setting API version based on a dynamic value (e.g. new Date()) may break your application at a random point in the future.
}
export const client = createClient(config)