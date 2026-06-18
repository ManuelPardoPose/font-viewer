import { invoke, Channel } from "@tauri-apps/api/core"

export type SimilarityEvent =
    | { event: "started"; data: { total: number } }
    | { event: "result"; data: { family: string; score: number | null } }
    | { event: "done" }
    | { event: "cancelled" }

let nextJobId = 1

export function newJobId(): number {
    return nextJobId++
}

export function findSimilarFonts(
    target: string,
    glyphs: string,
    weight: number,
    italic: boolean,
    jobId: number,
    onEvent: (event: SimilarityEvent) => void
): Promise<void> {
    const channel = new Channel<SimilarityEvent>()
    channel.onmessage = onEvent
    return invoke("find_similar_fonts", {
        target,
        glyphs,
        weight,
        italic,
        jobId,
        onEvent: channel,
    })
}

export function cancelSimilarity(jobId: number): Promise<void> {
    return invoke("cancel_similarity", { jobId })
}
