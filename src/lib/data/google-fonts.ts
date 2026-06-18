import { invoke } from "@tauri-apps/api/core"

// The metadata endpoint may guard its JSON with an XSSI prefix that must be
// stripped before parsing. The request itself is proxied through Rust because
// the endpoint doesn't send CORS headers (see fetch_google_fonts_metadata).
const XSSI_PREFIX = ")]}'"

interface GoogleFontMetadata {
    family: string
    fonts: Record<string, unknown>
    category: string
    popularity: number
    trending: number
    dateAdded: string
    subsets: string[]
}

interface MetadataResponse {
    familyMetadataList: GoogleFontMetadata[]
}

interface ParsedVariant {
    weight: number
    italic: boolean
}

function parseVariant(key: string): ParsedVariant | null {
    // Keys look like "400", "700i", "regular", "italic".
    const italic = key.endsWith("i") || key === "italic"
    const numeric = key.replace(/i$/, "")
    if (numeric === "regular" || numeric === "italic" || numeric === "") {
        return { weight: 400, italic }
    }
    const weight = Number.parseInt(numeric, 10)
    if (Number.isNaN(weight)) {
        return null
    }
    return { weight, italic }
}

function buildFamily(meta: GoogleFontMetadata): FontFamily {
    const fonts: Font[] = []
    for (const key of Object.keys(meta.fonts)) {
        const variant = parseVariant(key)
        if (!variant) {
            continue
        }
        fonts.push({
            family_name: meta.family,
            font_name: `${meta.family} ${variant.weight}${
                variant.italic ? " Italic" : ""
            }`,
            path: "",
            style: variant.italic ? "Italic" : "Normal",
            weight: variant.weight,
            stretch: 1,
        })
    }
    return {
        family_name: meta.family,
        fonts,
        source: "google",
        google: {
            category: meta.category,
            popularity: meta.popularity,
            trending: meta.trending,
            dateAdded: meta.dateAdded,
            // "menu" is an internal subset used for the family-name preview, not
            // a real language; drop it from the user-facing list.
            subsets: (meta.subsets ?? []).filter((s) => s !== "menu"),
        },
    }
}

let cache: Promise<FontFamily[]> | null = null

export function listGoogleFonts(): Promise<FontFamily[]> {
    if (!cache) {
        cache = invoke<string>("fetch_google_fonts_metadata")
            .then((body) => {
                const json = body.startsWith(XSSI_PREFIX)
                    ? body.slice(XSSI_PREFIX.length)
                    : body
                const data = JSON.parse(json) as MetadataResponse
                return data.familyMetadataList.map(buildFamily)
            })
            .catch((error) => {
                // Don't poison the cache: allow a later retry.
                cache = null
                throw error
            })
    }
    return cache
}

const loaded = new Map<string, Promise<void>>()

export function loadGoogleFont(family: FontFamily): Promise<void> {
    const existing = loaded.get(family.family_name)
    if (existing) {
        return existing
    }

    const normals = new Set<number>()
    const italics = new Set<number>()
    for (const font of family.fonts) {
        if (font.style === "Italic") {
            italics.add(font.weight)
        } else {
            normals.add(font.weight)
        }
    }

    let axis: string
    if (italics.size > 0) {
        const weights = [...new Set([...normals, ...italics])].sort(
            (a, b) => a - b
        )
        const tuples = [
            ...weights.map((w) => `0,${w}`),
            ...weights.filter((w) => italics.has(w)).map((w) => `1,${w}`),
        ]
        axis = `:ital,wght@${tuples.join(";")}`
    } else {
        const weights = [...normals].sort((a, b) => a - b)
        axis = `:wght@${weights.join(";")}`
    }

    const name = family.family_name.replace(/ /g, "+")
    const link = document.createElement("link")
    link.rel = "stylesheet"
    link.href = `https://fonts.googleapis.com/css2?family=${name}${axis}&display=swap`

    const ready = new Promise<void>((resolve) => {
        link.onload = () =>
            document.fonts
                .load(`16px "${family.family_name}"`)
                .then(() => resolve())
                .catch(() => resolve())
        link.onerror = () => resolve()
    })

    document.head.appendChild(link)
    loaded.set(family.family_name, ready)
    return ready
}

// Downloads the family's .ttf files and registers them with Windows.
// Returns the installed file paths.
export function installGoogleFont(family: FontFamily): Promise<string[]> {
    return invoke<string[]>("install_google_font", {
        family: family.family_name,
    })
}
