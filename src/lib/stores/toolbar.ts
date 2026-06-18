import { writable } from "svelte/store"

// Identifies which toolbar popover is currently open, so the size and weight
// popovers stay mutually exclusive.
export const openToolbarPopover = writable<string | null>(null)
