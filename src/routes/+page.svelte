<script lang="ts">
    import FontList from "$lib/components/font-list.svelte"

    import * as Sidebar from "$lib/components/ui/sidebar/index.js"
    import AppSidebar from "$lib/components/app-sidebar.svelte"

    import FontSearch from "$lib/components/font-search.svelte"

    import FontSizeSlider from "$lib/components/font-size-slider.svelte"
    import FontWeightSlider from "$lib/components/font-weight-slider.svelte"
    import FontStyleToggle from "$lib/components/font-style-toggle.svelte"

    import DarkModeToggle from "$lib/components/dark-mode-toggle.svelte"
    import { Separator } from "$lib/components/ui/separator/index.js"
</script>

<Sidebar.Provider class="max-w-full">
    <AppSidebar />
    <main class="flex h-screen w-full min-w-0 flex-col">
        <div
            class="flex items-center justify-between gap-4 border-b bg-background/80 p-2 backdrop-blur"
        >
            <div class="flex flex-1 items-center gap-2">
                <Sidebar.Trigger />
                <FontSearch />
            </div>

            <div class="flex items-center gap-1">
                <FontSizeSlider />
                <FontWeightSlider />
                <FontStyleToggle />
                <Separator orientation="vertical" class="mx-1 h-6" />
                <DarkModeToggle />
            </div>
        </div>
        <FontList />
    </main>
</Sidebar.Provider>
