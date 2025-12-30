import { defineConfig } from 'astro/config';

import tailwind from '@astrojs/tailwind';

// https://astro.build/config
export default defineConfig({
  image: {
    domains: ['sanity.io'],
    remotePatterns: [{ protocol: 'https' }],
  },
  output: 'static',
  integrations: [
    tailwind(),
  ],
});
