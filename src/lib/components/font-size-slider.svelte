<script lang="ts">
    import { Slider } from "$lib/components/ui/slider/index.js"
    import { LucideALargeSmall } from "lucide-svelte"
    import { previewOptions } from "$lib/stores/preview-options.js"
    import { openToolbarPopover } from "$lib/stores/toolbar.js"
    import * as Popover from "$lib/components/ui/popover/index.js"
    import { buttonVariants } from "$lib/components/ui/button/index.js"
</script>

<Popover.Root
    open={$openToolbarPopover === "size"}
    onOpenChange={(open) => openToolbarPopover.set(open ? "size" : null)}
>
    <Popover.Trigger class={buttonVariants({ variant: "ghost", size: "icon" })}
        ><LucideALargeSmall /></Popover.Trigger
    >
    <Popover.Content class="flex items-center gap-4 w-auto">
        <Slider
            value={[$previewOptions.fontSize]}
            min={4}
            max={200}
            step={1}
            class="w-48"
            onValueChange={(v) => {
                $previewOptions.fontSize = v[0]
            }}
        />
        <span class="w-12">{$previewOptions.fontSize}px</span>
    </Popover.Content>
</Popover.Root>
