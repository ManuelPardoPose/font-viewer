import { writable } from "svelte/store"

export interface FontStats {
    all: number
    system: number
    user: number
    google: number
    categories: Record<string, number>
    languages: Record<string, number>
}

// True while the Google Fonts catalogue is being fetched.
export const googleLoading = writable(false)

// Published by the font list so the sidebar can show filter counts without
// owning the font data itself.
export const fontStats = writable<FontStats>({
    all: 0,
    system: 0,
    user: 0,
    google: 0,
    categories: {},
    languages: {},
})
