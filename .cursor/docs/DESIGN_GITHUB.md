# GitHub README theming (Primer)

Colors below match GitHub’s **default `light` and `dark`** themes from **[@primer/primitives](https://www.npmjs.com/package/@primer/primitives)** (`dist/css/functional/themes/light.css` and `dark.css`). Use them when designing README hero images that should sit on the rendered README canvas.

## Main content area (body text + canvas)


| Mode  | Role                          | CSS variable (semantic) | Hex       |
| ----- | ----------------------------- | ----------------------- | --------- |
| Light | Default text (headings, body) | `--fgColor-default`     | `#1f2328` |
| Light | Page / content background     | `--bgColor-default`     | `#ffffff` |
| Dark  | Default text                  | `--fgColor-default`     | `#f0f6fc` |
| Dark  | Page / content background     | `--bgColor-default`     | `#0d1117` |


## Nearby surfaces (light mode)

The **file header** strip (e.g. “README.md” above the content) often uses a muted surface such as `**#f6f8fa`** (`--bgColor-muted` / related control tokens), not `#ffffff`. Do not assume the whole viewport is one flat white.

## Eyedropper vs tokens

- Published Primer **dark** foreground is `**#f0f6fc`**. Some screenshots or older notes show `**#e6edf3**`; it is close but not the current `fgColor-default` in default `dark`.

## Theme-aware hero images

GitHub-rendered Markdown (including `README.md`) supports the HTML `**<picture>**` element ([Basic writing — The Picture element](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#the-picture-element)). Use `**prefers-color-scheme**` media on `<source>` lines so viewers get a PNG tuned for **dark** vs **light** canvas and text contrast (see hex table above). Keep repo assets on **paths relative to that README file** so clones and IA-style copies resolve without a CDN.

```html
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/your-banner-dark.png">
  <source media="(prefers-color-scheme: light)" srcset="images/your-banner-light.png">
  <img alt="Short descriptive alt text" src="images/your-banner-light.png">
</picture>
```

The `**img**` fallback should match one of your sources (commonly light) for environments that ignore `<picture>`. Use meaningful `**alt**` text; plain Markdown `![…](…)` alone cannot toggle two banners by theme.

Selection follows the browser `**prefers-color-scheme**` signal, which usually tracks **Appearance → Sync with system**. If someone forces light GitHub UI while OS stays dark (or vice versa), the chosen PNG can mismatch the GitHub chrome—still the supported pattern for Markdown; see discussions in [primer/markup issue #1583](https://github.com/github/markup/issues/1583).

## Further reading

- [GitHub Docs — Basic Markdown: The Picture element](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#the-picture-element)
- [GitHub Blog — README images for dark and light mode (picture + prefers-color-scheme)](https://github.blog/developer-skills/github/how-to-make-your-images-in-markdown-on-github-adjust-for-dark-mode-and-light-mode/)
- [Primer primitives — getting started](https://primer.style/foundations/primitives/getting-started)
- [primer/primitives README](https://github.com/primer/primitives/blob/main/README.md) (theme CSS import paths)

