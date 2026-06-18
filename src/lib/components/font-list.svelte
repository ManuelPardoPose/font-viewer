<script lang="ts">
    import { ScrollArea } from "$lib/components/ui/scroll-area/index.js"
    import { Separator } from "$lib/components/ui/separator/index.js"

    import { invoke } from "@tauri-apps/api/core"

    import FamilyPreview from "./family-preview.svelte"

    import { previewOptions } from "$lib/stores/preview-options"
    import {
        fontFilters,
        FontSource,
        FontCategory,
        FontSort,
    } from "$lib/stores/font-filters"
    import { listGoogleFonts } from "$lib/data/google-fonts"
    import { pinnedFonts } from "$lib/stores/pins"
    import { fontStats, googleLoading } from "$lib/stores/font-stats"
    import {
        findSimilarFonts,
        cancelSimilarity,
        newJobId,
    } from "$lib/data/similarity"
    import { LoaderCircle, X } from "lucide-svelte"
    import { tick, untrack } from "svelte"
    import { get } from "svelte/store"

    let scrollElement = $state<HTMLElement | null>(null)

    let installed_families = $state<FontFamily[]>([])
    let google_families = $state<FontFamily[]>([])

    let installed_paths = $derived(
        new Map(
            installed_families.flatMap((f) => {
                const path = f.fonts.find((font) => font.path.length > 0)?.path
                return path ? [[f.family_name.toLowerCase(), path]] : []
            })
        )
    )

    let source_families = $derived(
        $fontFilters.source === FontSource.GoogleFonts
            ? google_families
            : installed_families
    )

    $effect(() => {
        const categories: Record<string, number> = {}
        const languages: Record<string, number> = {}
        for (const family of google_families) {
            const category = family.google?.category
            if (category) {
                categories[category] = (categories[category] ?? 0) + 1
            }
            for (const subset of family.google?.subsets ?? []) {
                languages[subset] = (languages[subset] ?? 0) + 1
            }
        }
        fontStats.set({
            all: installed_families.length,
            system: installed_families.filter((f) =>
                f.fonts.some((font) =>
                    font.path.toLowerCase().includes("c:\\windows")
                )
            ).length,
            user: installed_families.filter((f) =>
                f.fonts.some((font) =>
                    font.path.toLowerCase().includes("c:\\users")
                )
            ).length,
            google: google_families.length,
            categories,
            languages,
        })
    })

    let filtered_families = $derived(
        source_families.filter((family) => {
            switch ($fontFilters.source) {
                case FontSource.System:
                    if (
                        !family.fonts.some((font) =>
                            font.path.toLowerCase().includes("c:\\windows")
                        )
                    ) {
                        return false
                    }
                    break
                case FontSource.User:
                    if (
                        !family.fonts.some((font) =>
                            font.path.toLowerCase().includes("c:\\users")
                        )
                    ) {
                        return false
                    }
                    break
                case FontSource.GoogleFonts:
                    if (
                        $fontFilters.category !== FontCategory.All &&
                        family.google?.category !== $fontFilters.category
                    ) {
                        return false
                    }
                    if (
                        $fontFilters.language !== "" &&
                        !family.google?.subsets.includes($fontFilters.language)
                    ) {
                        return false
                    }
                    break
            }
            return family.family_name
                .toLowerCase()
                .includes($fontFilters.search.toLowerCase())
        })
    )

    let sorted_families = $derived(
        $fontFilters.source !== FontSource.GoogleFonts
            ? filtered_families
            : [...filtered_families].sort((a, b) => {
                  switch ($fontFilters.sort) {
                      case FontSort.Trending:
                          return (
                              (a.google?.trending ?? Infinity) -
                              (b.google?.trending ?? Infinity)
                          )
                      case FontSort.Popular:
                          return (
                              (a.google?.popularity ?? Infinity) -
                              (b.google?.popularity ?? Infinity)
                          )
                      case FontSort.Newest:
                          return (b.google?.dateAdded ?? "").localeCompare(
                              a.google?.dateAdded ?? ""
                          )
                      case FontSort.Name:
                          return a.family_name.localeCompare(b.family_name)
                  }
              })
    )

    let pinned_families = $derived(
        sorted_families.filter((f) => $pinnedFonts.includes(f.family_name))
    )
    let unpinned_families = $derived(
        sorted_families.filter((f) => !$pinnedFonts.includes(f.family_name))
    )

    let containerEl = $state<HTMLElement | null>(null)
    let pinnedHeight = $state(240)

    // Measure the natural height of the pinned rows so the panel never reserves
    // more space than its content needs.
    let pinnedContentEl = $state<HTMLElement | null>(null)
    let pinnedContentHeight = $state(0)
    let pinnedRowHeight = $state(0)

    $effect(() => {
        const el = pinnedContentEl
        if (!el) return
        const measure = () => {
            pinnedContentHeight = el.scrollHeight
            pinnedRowHeight =
                (el.firstElementChild as HTMLElement | null)?.offsetHeight ?? 0
        }
        const observer = new ResizeObserver(measure)
        observer.observe(el)
        measure()
        return () => observer.disconnect()
    })

    let effectivePinnedHeight = $derived(
        pinnedContentHeight > 0
            ? Math.min(pinnedHeight, pinnedContentHeight)
            : pinnedHeight
    )

    function startResize(event: PointerEvent) {
        event.preventDefault()
        const startY = event.clientY
        const startHeight = pinnedHeight
        const min = pinnedRowHeight || 80
        const max = Math.max(min, (containerEl?.clientHeight ?? 600) - 120)

        function move(e: PointerEvent) {
            const next = startHeight + (e.clientY - startY)
            pinnedHeight = Math.max(min, Math.min(next, max))
        }
        function up() {
            window.removeEventListener("pointermove", move)
            window.removeEventListener("pointerup", up)
        }
        window.addEventListener("pointermove", move)
        window.addEventListener("pointerup", up)
    }

    // --- Similarity comparison ---------------------------------------------
    let similarityTarget = $state<string | null>(null)
    let similarityDone = $state(0)
    let similarityTotal = $state(0)
    let similarityScores = $state<Map<string, number>>(new Map())

    // Raw incoming results, flushed into the reactive state on a timer so the
    // list doesn't re-sort on every streamed message.
    let pendingScores = new Map<string, number>()
    let pendingDone = 0
    let flushTimer: ReturnType<typeof setTimeout> | undefined
    let currentJob = 0

    function flushSimilarity() {
        flushTimer = undefined
        similarityScores = new Map(pendingScores)
        similarityDone = pendingDone
    }

    function startSimilar(family: string) {
        if (similarityTarget) {
            cancelSimilarity(currentJob)
        }
        currentJob = newJobId()
        similarityTarget = family
        pendingScores = new Map()
        pendingDone = 0
        similarityScores = new Map()
        similarityDone = 0
        similarityTotal = 0

        const job = currentJob
        const options = get(previewOptions)
        findSimilarFonts(
            family,
            options.text,
            options.fontWeight,
            options.fontStyle === "italic",
            job,
            (event) => {
            if (job !== currentJob) return
            switch (event.event) {
                case "started":
                    similarityTotal = event.data.total
                    break
                case "result":
                    if (event.data.score !== null) {
                        pendingScores.set(event.data.family, event.data.score)
                    }
                    pendingDone += 1
                    if (!flushTimer) {
                        flushTimer = setTimeout(flushSimilarity, 100)
                    }
                    break
                case "done":
                case "cancelled":
                    flushSimilarity()
                    break
            }
        })
    }

    function stopSimilar() {
        if (similarityTarget) {
            cancelSimilarity(currentJob)
        }
        currentJob = 0
        similarityTarget = null
    }

    // Leaving the similarity view when the user switches source keeps things
    // unambiguous.
    $effect(() => {
        $fontFilters.source
        untrack(() => {
            if (similarityTarget) {
                stopSimilar()
            }
        })
    })

    let similar_target_family = $derived(
        similarityTarget
            ? installed_families.find((f) => f.family_name === similarityTarget)
            : undefined
    )
    let similar_results = $derived(
        [...similarityScores.entries()]
            .sort((a, b) => b[1] - a[1])
            .flatMap(([name, score]) => {
                const family = installed_families.find(
                    (f) => f.family_name === name
                )
                return family ? [{ family, score }] : []
            })
    )
    let similarityProgress = $derived(
        similarityTotal > 0
            ? Math.min(100, (similarityDone / similarityTotal) * 100)
            : 0
    )

    let googleError = $state(false)

    $effect(() => {
        if (
            $fontFilters.source === FontSource.GoogleFonts &&
            google_families.length === 0 &&
            !$googleLoading
        ) {
            googleLoading.set(true)
            googleError = false
            listGoogleFonts()
                .then((families) => (google_families = families))
                .catch((error) => {
                    googleError = true
                    console.error("Failed to load Google Fonts", error)
                })
                .finally(() => googleLoading.set(false))
        }
    })

    previewOptions.subscribe(async (options) => {
        if (scrollElement) {
            let scrollPercentage =
                scrollElement.scrollTop / scrollElement.scrollHeight
            await tick()
            scrollElement.scrollTop =
                scrollPercentage * scrollElement.scrollHeight
        }
    })

    function loadInstalledFonts() {
        invoke<Font[]>("list_fonts").then((fonts) => {
            const families: FontFamily[] = []
            fonts.forEach((font) => {
                const family = families.find(
                    (f) => f.family_name === font.family_name
                )
                if (family) {
                    family.fonts.push(font)
                } else {
                    families.push({
                        family_name: font.family_name,
                        fonts: [font],
                    })
                }
            })
            installed_families = families
        })
    }

    loadInstalledFonts()

    // DirectWrite caches its font collection per-process, so a font installed
    // this session won't appear on re-enumeration. Merge it into the local
    // list directly using the family metadata and the installed file paths.
    function registerInstalled(family: FontFamily, paths: string[]) {
        const fonts: Font[] = family.fonts.map((font, i) => ({
            ...font,
            path: paths[i] ?? paths[0] ?? "",
        }))
        const existing = installed_families.find(
            (f) => f.family_name === family.family_name
        )
        if (existing) {
            existing.fonts = fonts
            installed_families = [...installed_families]
        } else {
            installed_families = [
                ...installed_families,
                { family_name: family.family_name, fonts },
            ]
        }
    }
</script>

{#snippet group(list: FontFamily[])}
    {#each list as family, i (family.family_name)}
        {#if i > 0}
            <Separator />
        {/if}
        <FamilyPreview
            {family}
            {installed_paths}
            oninstalled={(paths) => registerInstalled(family, paths)}
            onfindsimilar={() => startSimilar(family.family_name)}
        />
    {/each}
{/snippet}

<div bind:this={containerEl} class="flex min-h-0 flex-1 flex-col">
    {#if similarityTarget}
        <div class="relative overflow-hidden border-b px-5 py-2 text-sm">
            <div
                class="absolute inset-y-0 left-0 bg-accent transition-[width] duration-200 ease-out"
                style="width: {similarityProgress}%"
            ></div>
            <div class="relative flex items-center gap-2">
                <span>
                    Similar to
                    <span class="font-semibold">{similarityTarget}</span>
                </span>
                <span class="tabular-nums text-muted-foreground">
                    {similarityDone}/{similarityTotal}
                </span>
                {#if similarityTotal === 0 || similarityDone < similarityTotal}
                    <LoaderCircle
                        class="size-4 animate-spin text-muted-foreground"
                    />
                {/if}
                <button
                    onclick={stopSimilar}
                    title="Close comparison"
                    class="ml-auto flex items-center gap-1 text-muted-foreground transition-colors hover:text-foreground"
                >
                    <X class="size-4" />
                    Close
                </button>
            </div>
        </div>
        <div class="min-h-0 flex-1">
            <ScrollArea orientation="vertical" class="h-full">
                {#if similar_target_family}
                    <FamilyPreview
                        family={similar_target_family}
                        {installed_paths}
                    />
                    <Separator />
                {/if}
                {#each similar_results as item, i (item.family.family_name)}
                    {#if i > 0}
                        <Separator />
                    {/if}
                    <FamilyPreview
                        family={item.family}
                        {installed_paths}
                        score={item.score}
                        onfindsimilar={() =>
                            startSimilar(item.family.family_name)}
                    />
                {/each}
            </ScrollArea>
        </div>
    {:else if $googleLoading && sorted_families.length === 0}
        <div
            class="flex items-center gap-2 px-5 py-4 text-sm text-muted-foreground"
        >
            <LoaderCircle class="size-4 animate-spin" />
            Loading Google Fonts...
        </div>
    {:else if googleError && sorted_families.length === 0}
        <div class="px-5 py-4 text-sm text-muted-foreground">
            Couldn't load Google Fonts. Check your connection and try again.
        </div>
    {:else if sorted_families.length === 0}
        <div class="px-5 py-4 text-sm text-muted-foreground">No fonts found</div>
    {:else}
        {#if pinned_families.length > 0}
            <div
                class="shrink-0 overflow-hidden"
                style="height: {effectivePinnedHeight}px"
            >
                <ScrollArea orientation="vertical" class="h-full">
                    <div bind:this={pinnedContentEl}>
                        {@render group(pinned_families)}
                    </div>
                </ScrollArea>
            </div>
            <div
                role="separator"
                aria-orientation="horizontal"
                tabindex="-1"
                title="Drag to resize the pinned area"
                class="group/handle flex h-2 shrink-0 cursor-row-resize items-center justify-center border-y bg-border/30 transition-colors hover:bg-border"
                onpointerdown={startResize}
            >
                <div
                    class="h-0.5 w-8 rounded-full bg-muted-foreground/40 transition-colors group-hover/handle:bg-muted-foreground"
                ></div>
            </div>
        {/if}
        <div class="min-h-0 flex-1">
            <ScrollArea
                bind:viewportRef={scrollElement}
                orientation="vertical"
                class="h-full"
            >
                {@render group(unpinned_families)}
            </ScrollArea>
        </div>
    {/if}
</div>
