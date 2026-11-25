# Wincy website

The project page for [Wincy](../README.md), built with [Astro](https://astro.build) and published to GitHub Pages.
English lives at `/`, Spanish at `/es/`.

## Develop

```sh
cd website
pnpm install
pnpm dev       # http://localhost:4321/wincy/
pnpm build     # type-checks (astro check), then writes static files to dist/
pnpm preview   # serves dist/
```

## Where things are

| path | what |
|---|---|
| `src/i18n/en.ts`, `src/i18n/es.ts` | every user-facing string, including the demo clips. `es.ts` is type-checked against `en.ts`, so a missing key fails the build. |
| `src/components/Landing.astro` | all page sections, shared by both locales |
| `src/components/Palette.astro` | the interactive palette demo (vanilla TS, no framework) |
| `src/pages/index.astro`, `src/pages/es/index.astro` | one route per locale |
| `src/styles/global.css` | design tokens (light and dark) and all styles |
| `public/` | `logo.svg` (copy of `assets/logo.svg`, also the favicon) and `og.png` (social preview) |

To add a language, create `src/i18n/<code>.ts`, register it in `src/i18n/index.ts`, add it to `i18n.locales`
and the sitemap `locales` in `astro.config.mjs`, and add `src/pages/<code>/index.astro`.

## Changing the owner or the base path

The site is configured as a GitHub project page at `https://devnielo.github.io/wincy/`:

- `astro.config.mjs`: `site` (`https://<owner>.github.io`) and `base` (`/<repo-name>`).
- `src/consts.ts`: the repository URL used for the download and GitHub links.

If the repository is not named `wincy`, change `base` to match. For a custom domain or a
`<owner>.github.io` user site, set `site` to that URL and remove `base`.

## Deploying

`.github/workflows/pages.yml` builds and deploys on every push to `main` that touches `website/`,
and can be run by hand from the Actions tab. In the repository settings, set **Pages → Source** to
**GitHub Actions** once.
