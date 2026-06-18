import { writable } from "svelte/store"

export enum FontSource {
    All = "All Fonts",
    System = "System Fonts",
    User = "User Fonts",
    GoogleFonts = "Google Fonts",
}

export enum FontCategory {
    All = "All Categories",
    SansSerif = "Sans Serif",
    Serif = "Serif",
    Display = "Display",
    Handwriting = "Handwriting",
    Monospace = "Monospace",
}

export enum FontSort {
    Trending = "Trending",
    Popular = "Most popular",
    Newest = "Newest",
    Name = "Name",
}

interface FontFilters {
    search: string
    source: FontSource
    category: FontCategory
    sort: FontSort
    language: string
}

export const ALL_LANGUAGES = ""

const defaultFilters: FontFilters = {
    search: "",
    source: FontSource.All,
    category: FontCategory.All,
    sort: FontSort.Trending,
    language: ALL_LANGUAGES,
}

export const fontFilters = writable<FontFilters>({ ...defaultFilters })

export function resetFilters() {
    fontFilters.set({ ...defaultFilters })
}
