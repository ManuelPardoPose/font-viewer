<script lang="ts">
    import { Italic } from "lucide-svelte"
    import { previewOptions } from "$lib/stores/preview-options.js"
    import { Toggle } from "$lib/components/ui/toggle/index.js"

    const toggleStyle = () => {
        $previewOptions.fontStyle =
            $previewOptions.fontStyle === "italic" ? "normal" : "italic"
    }
</script>

<Toggle
    pressed={$previewOptions.fontStyle === "italic"}
    onPressedChange={toggleStyle}
>
    <Italic />
</Toggle>
