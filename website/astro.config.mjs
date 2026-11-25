// @ts-check
import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";

// Project page on GitHub Pages: https://devnielo.github.io/wincy/ (see README.md to change the owner or base).
export default defineConfig({
  site: "https://devnielo.github.io",
  base: "/wincy",
  i18n: {
    defaultLocale: "en",
    locales: ["en", "es"],
    routing: { prefixDefaultLocale: false },
  },
  integrations: [
    sitemap({
      i18n: { defaultLocale: "en", locales: { en: "en", es: "es" } },
    }),
  ],
});
