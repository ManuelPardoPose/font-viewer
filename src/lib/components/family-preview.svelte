<script lang="ts">
    import { previewOptions } from "$lib/stores/preview-options.js"
    import { loadGoogleFont, installGoogleFont } from "$lib/data/google-fonts"
    import { revealItemInDir } from "@tauri-apps/plugin-opener"
    import { pinnedFonts, togglePin } from "$lib/stores/pins.js"
    import { inview } from "svelte-inview"
    import {
        Download,
        Check,
        LoaderCircle,
        TriangleAlert,
        FolderOpen,
        Pin,
        ScanSearch,
    } from "lucide-svelte"

    interface Props {
        family: FontFamily
        installed_paths?: Map<string, string>
        oninstalled?: (paths: string[]) => void
        onfindsimilar?: () => void
        score?: number
    }

    let { family, installed_paths, oninstalled, onfindsimilar, score }: Props =
        $props()

    let isInView = $state(false)
    let fontReady = $state(family.source !== "google")

    // Path captured from a just-completed install, before the local font
    // list is aware of the new file.
    let installedPath = $state<string | undefined>()

    // The path to reveal: a local font carries its own path; an installed
    // Google font matches an entry in the installed list by family name.
    let fontPath = $derived(
        family.fonts.find((font) => font.path.length > 0)?.path ??
            installed_paths?.get(family.family_name.toLowerCase()) ??
            installedPath
    )

    let isPinned = $derived($pinnedFonts.includes(family.family_name))

    let copied = $state(false)
    let copyTimeout: ReturnType<typeof setTimeout> | undefined

    function copyName() {
        navigator.clipboard.writeText(family.family_name).then(() => {
            copied = true
            clearTimeout(copyTimeout)
            copyTimeout = setTimeout(() => (copied = false), 1200)
        })
    }

    function revealInExplorer() {
        if (fontPath) {
            revealItemInDir(fontPath).catch((error) =>
                console.error("Failed to reveal font file", error)
            )
        }
    }

    type InstallState = "idle" | "installing" | "done" | "error"
    let installState = $state<InstallState>("idle")

    let isInstalled = $derived(
        installState === "done" ||
            (installed_paths?.has(family.family_name.toLowerCase()) ?? false)
    )

    $effect(() => {
        if (isInView && family.source === "google" && !fontReady) {
            loadGoogleFont(family).then(() => (fontReady = true))
        }
    })

    async function install() {
        if (installState === "installing" || installState === "done") {
            return
        }
        installState = "installing"
        try {
            const paths = await installGoogleFont(family)
            installedPath = paths[0]
            installState = "done"
            oninstalled?.(paths)
        } catch (error) {
            console.error("Failed to install font", error)
            installState = "error"
        }
    }
</script>

<div class="group px-5 py-3 transition-colors hover:bg-accent/40">
    <div class="flex items-center gap-2 text-sm">
        <button
            onclick={() => togglePin(family.family_name)}
            title={isPinned ? "Unpin" : "Pin"}
            class="text-muted-foreground transition-all hover:text-foreground focus-visible:opacity-100
                {isPinned ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}"
        >
            <Pin class="size-4" fill={isPinned ? "currentColor" : "none"} />
        </button>
        <button
            onclick={copyName}
            title="Click to copy name"
            class="font-semibold hover:underline"
        >
            {family.family_name}
        </button>
        <span
            class="rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground tabular-nums"
            title="{family.fonts.length} style{family.fonts.length === 1
                ? ''
                : 's'}"
        >
            {family.fonts.length}
        </span>
        {#if family.source === "google" && family.google}
            <span class="text-xs text-muted-foreground">
                {family.google.category}
            </span>
        {/if}
        {#if copied}
            <span class="flex items-center gap-1 text-xs text-muted-foreground">
                <Check class="size-3" />
                Copied
            </span>
        {/if}

        <div class="ml-auto flex items-center gap-3">
            {#if score !== undefined}
                <span
                    class="tabular-nums text-sm font-medium text-muted-foreground"
                >
                    {Math.round(score)}%
                </span>
            {/if}
            <div class="flex items-center gap-1">
                {#if family.source !== "google" && onfindsimilar}
                    <button
                        onclick={onfindsimilar}
                        title="Find similar fonts"
                        class="text-muted-foreground opacity-0 transition-all hover:text-foreground focus-visible:opacity-100 group-hover:opacity-100"
                    >
                        <ScanSearch class="size-4" />
                    </button>
                {/if}
                {#if family.source === "google"}
                    {#if isInstalled && fontPath}
                        <button
                            onclick={revealInExplorer}
                            title="Installed - show in Explorer"
                            class="text-muted-foreground transition-colors hover:text-foreground"
                        >
                            <Check class="size-4 group-hover:hidden" />
                            <FolderOpen
                                class="hidden size-4 group-hover:block"
                            />
                        </button>
                    {:else if isInstalled}
                        <span class="text-muted-foreground" title="Installed">
                            <Check class="size-4" />
                        </span>
                    {:else}
                        <button
                            onclick={install}
                            disabled={installState === "installing"}
                            title={installState === "error"
                                ? "Install failed - click to retry"
                                : "Install to Windows"}
                            class="text-muted-foreground transition-all hover:text-foreground focus-visible:opacity-100 disabled:opacity-100
                                {installState === 'idle'
                                ? 'opacity-0 group-hover:opacity-100'
                                : 'opacity-100'}"
                        >
                            {#if installState === "installing"}
                                <LoaderCircle class="size-4 animate-spin" />
                            {:else if installState === "error"}
                                <TriangleAlert class="size-4" />
                            {:else}
                                <Download class="size-4" />
                            {/if}
                        </button>
                    {/if}
                {:else if fontPath}
                    <button
                        onclick={revealInExplorer}
                        title="Show in Explorer"
                        class="text-muted-foreground opacity-0 transition-all hover:text-foreground focus-visible:opacity-100 group-hover:opacity-100"
                    >
                        <FolderOpen class="size-4" />
                    </button>
                {/if}
            </div>
        </div>
    </div>
    <div
        use:inview={{ rootMargin: "10%" }}
        oninview_change={({ detail }) => (isInView = detail.inView)}
        style="height: {$previewOptions.fontSize * 1.4}px;"
        class="mt-1"
    >
        <input
            style="
            font-family: {family.family_name};
            font-size: {$previewOptions.fontSize}px;
            font-weight: {$previewOptions.fontWeight};
            font-style: {$previewOptions.fontStyle};
            display: {isInView ? 'block' : 'none'};
            opacity: {fontReady ? 1 : 0};
            transition: opacity 150ms ease;
        "
            class="h-full w-full whitespace-nowrap bg-transparent leading-none outline-none"
            bind:value={$previewOptions.text}
            type="text"
        />
    </div>
</div>
