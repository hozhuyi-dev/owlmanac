/// <reference types="vitest/config" />
import { getViteConfig } from "astro/config";

// Tests run with the site's own Vite setup (aliases, Astro's plugins), beside the file they test:
// x.ts → x.test.ts. They cover logic that runs without a browser; npm run build runs them first.
export default getViteConfig({
  test: {
    include: ["src/**/*.test.ts"],
  },
});
