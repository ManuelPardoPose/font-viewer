<script lang="ts">
    import { Input } from "$lib/components/ui/input/index.js"
    import { Search } from "lucide-svelte"
    import { fontFilters } from "$lib/stores/font-filters.js"
</script>

<div class="relative w-full">
    <Search
        class="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
    />
    <Input
        type="text"
        bind:value={$fontFilters.search}
        placeholder="Search fonts..."
        class="pl-8"
    />
</div>