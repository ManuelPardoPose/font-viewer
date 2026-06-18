<script lang="ts">
    import * as Sidebar from "$lib/components/ui/sidebar/index.js"
    import { ScrollArea } from "$lib/components/ui/scroll-area/index.js"
    import { buttonVariants } from "$lib/components/ui/button/index.js"
    import { RotateCcw, LoaderCircle } from "lucide-svelte"

    import {
        fontFilters,
        FontSource,
        FontCategory,
        FontSort,
        ALL_LANGUAGES,
        resetFilters,
    } from "$lib/stores/font-filters"
    import { fontStats, googleLoading } from "$lib/stores/font-stats"

    let isGoogle = $derived($fontFilters.source === FontSource.GoogleFonts)

    function sourceCount(source: FontSource): number {
        switch (source) {
            case FontSource.All:
                return $fontStats.all
            case FontSource.System:
                return $fontStats.system
            case FontSource.User:
                return $fontStats.user
            case FontSource.GoogleFonts:
                return $fontStats.google
        }
    }

    let languageList = $derived(
        Object.entries($fontStats.languages)
            .sort((a, b) => b[1] - a[1])
            .slice(0, 12)
    )

    function languageLabel(code: string): string {
        return code
            .split("-")
            .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
            .join(" ")
    }
</script>

<Sidebar.Root>
    <Sidebar.Header>
        <Sidebar.Group class="py-0">
            <Sidebar.GroupLabel>Source</Sidebar.GroupLabel>
            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    {#each Object.values(FontSource) as source}
                        <Sidebar.MenuItem>
                            <Sidebar.MenuButton
                                isActive={$fontFilters.source === source}
                                onclick={() => ($fontFilters.source = source)}
                            >
                                <span>{source}</span>
                            </Sidebar.MenuButton>
                            {#if source === FontSource.GoogleFonts && $googleLoading}
                                <Sidebar.MenuBadge>
                                    <LoaderCircle class="size-3 animate-spin" />
                                </Sidebar.MenuBadge>
                            {:else if sourceCount(source) > 0}
                                <Sidebar.MenuBadge>
                                    {sourceCount(source)}
                                </Sidebar.MenuBadge>
                            {/if}
                        </Sidebar.MenuItem>
                    {/each}
                </Sidebar.Menu>
            </Sidebar.GroupContent>
        </Sidebar.Group>
    </Sidebar.Header>

    <Sidebar.Content class="overflow-hidden">
        <ScrollArea orientation="vertical" class="h-full">
            <div class="flex flex-col gap-2">
        {#if isGoogle}
            <Sidebar.Group>
                <Sidebar.GroupLabel>Category</Sidebar.GroupLabel>
                <Sidebar.GroupContent>
                    <Sidebar.Menu>
                        {#each Object.values(FontCategory) as category}
                            <Sidebar.MenuItem>
                                <Sidebar.MenuButton
                                    isActive={$fontFilters.category === category}
                                    onclick={() =>
                                        ($fontFilters.category = category)}
                                >
                                    <span>{category}</span>
                                </Sidebar.MenuButton>
                                {#if category !== FontCategory.All && $fontStats.categories[category]}
                                    <Sidebar.MenuBadge>
                                        {$fontStats.categories[category]}
                                    </Sidebar.MenuBadge>
                                {/if}
                            </Sidebar.MenuItem>
                        {/each}
                    </Sidebar.Menu>
                </Sidebar.GroupContent>
            </Sidebar.Group>

            <Sidebar.Group>
                <Sidebar.GroupLabel>Sort</Sidebar.GroupLabel>
                <Sidebar.GroupContent>
                    <Sidebar.Menu>
                        {#each Object.values(FontSort) as sort}
                            <Sidebar.MenuItem>
                                <Sidebar.MenuButton
                                    isActive={$fontFilters.sort === sort}
                                    onclick={() => ($fontFilters.sort = sort)}
                                >
                                    <span>{sort}</span>
                                </Sidebar.MenuButton>
                            </Sidebar.MenuItem>
                        {/each}
                    </Sidebar.Menu>
                </Sidebar.GroupContent>
            </Sidebar.Group>

            {#if languageList.length > 0}
                <Sidebar.Group>
                    <Sidebar.GroupLabel>Language</Sidebar.GroupLabel>
                    <Sidebar.GroupContent>
                        <Sidebar.Menu>
                            <Sidebar.MenuItem>
                                <Sidebar.MenuButton
                                    isActive={$fontFilters.language ===
                                        ALL_LANGUAGES}
                                    onclick={() =>
                                        ($fontFilters.language = ALL_LANGUAGES)}
                                >
                                    <span>All Languages</span>
                                </Sidebar.MenuButton>
                            </Sidebar.MenuItem>
                            {#each languageList as [code, count]}
                                <Sidebar.MenuItem>
                                    <Sidebar.MenuButton
                                        isActive={$fontFilters.language === code}
                                        onclick={() =>
                                            ($fontFilters.language = code)}
                                    >
                                        <span>{languageLabel(code)}</span>
                                    </Sidebar.MenuButton>
                                    <Sidebar.MenuBadge>{count}</Sidebar.MenuBadge>
                                </Sidebar.MenuItem>
                            {/each}
                        </Sidebar.Menu>
                    </Sidebar.GroupContent>
                </Sidebar.Group>
            {/if}
        {/if}

            </div>
        </ScrollArea>
    </Sidebar.Content>

    <Sidebar.Footer>
        <div class="px-2 py-1">
            <button
                onclick={resetFilters}
                class={buttonVariants({ variant: "ghost", size: "sm" })}
            >
                <RotateCcw class="size-4" />
                Reset filters
            </button>
        </div>
    </Sidebar.Footer>
</Sidebar.Root>
