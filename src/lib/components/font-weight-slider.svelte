<script lang="ts">
    import { Slider } from "$lib/components/ui/slider/index.js"
    import { Bold } from "lucide-svelte"
    import { previewOptions } from "$lib/stores/preview-options.js"
    import { openToolbarPopover } from "$lib/stores/toolbar.js"
    import * as Popover from "$lib/components/ui/popover/index.js"
    import { buttonVariants } from "$lib/components/ui/button/index.js"
</script>

<Popover.Root
    open={$openToolbarPopover === "weight"}
    onOpenChange={(open) => openToolbarPopover.set(open ? "weight" : null)}
>
    <Popover.Trigger class={buttonVariants({ variant: "ghost", size: "icon" })}
        ><Bold /></Popover.Trigger
    >
    <Popover.Content class="flex items-center gap-4 w-auto">
        <Slider
            value={[$previewOptions.fontWeight]}
            min={1}
            max={1000}
            step={1}
            class="w-48"
            onValueChange={(v) => {
                $previewOptions.fontWeight = v[0]
            }}
        />
        <span class="w-10">{$previewOptions.fontWeight}</span>
    </Popover.Content>
</Popover.Root>
