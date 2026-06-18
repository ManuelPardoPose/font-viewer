# Font Viewer

<img width="50%" src="./src-tauri/icons/app-icon.png">

A lightweight font viewer app built with Tauri, SvelteKit, TypeScript, and Tailwind CSS.

## Features

- Modern UI with dark & light mode
- Fast and lightweight
- View all installed fonts on your system
- Filter fonts by name & source (System/User)
- Browse Google Fonts and install them with one click
- Filter Google Fonts by category, language, and sort order
- Pin fonts to the top to compare them against others as you scroll
- Customize font size, weight, and style
- Custom text preview
- Copy a font's name or reveal its file in the explorer

## Wishlist

- Rasterize fonts in backend using [fontdue](https://github.com/mooman219/fontdue)
- Compare fonts (glyphs & custom text) using [dssim](https://github.com/kornelski/dssim)
- Sort fonts by similarity
- Expand a family to preview every weight & style at once
- Variable font axis controls (weight, width, slant, …)
- Uninstall fonts from within the app
- Remember preview settings & filters between sessions

## Screenshots

![Screenshot 1](https://github.com/user-attachments/assets/214d0287-e981-4cff-8c48-c71401b63e30)

![Screenshot 2](https://github.com/user-attachments/assets/4b7adfd3-f126-40fa-9895-8fec1e140240)

![Screenshot 3](https://github.com/user-attachments/assets/ea5c5482-3eea-401c-bff0-f9c3777e4838)

## Build

```bash
pnpm i

pnpm tauri dev
```
