import { writable } from "svelte/store"

const KEY = "pinnedFonts"

function load(): string[] {
    if (typeof localStorage === "undefined") {
        return []
    }
    try {
        const stored = JSON.parse(localStorage.getItem(KEY) ?? "[]")
        return Array.isArray(stored) ? stored : []
    } catch {
        return []
    }
}

// Pinned font family names, persisted across sessions.
export const pinnedFonts = writable<string[]>(load())

pinnedFonts.subscribe((pins) => {
    if (typeof localStorage !== "undefined") {
        localStorage.setItem(KEY, JSON.stringify(pins))
    }
})

export function togglePin(name: string) {
    pinnedFonts.update((pins) =>
        pins.includes(name) ? pins.filter((p) => p !== name) : [...pins, name]
    )
}
