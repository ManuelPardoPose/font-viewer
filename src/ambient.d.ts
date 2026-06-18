declare global {
    interface Font {
        family_name: string
        font_name: string
        path: string
        style: string
        weight: number
        stretch: number
    }

    interface GoogleMetadata {
        category: string
        popularity: number
        trending: number
        dateAdded: string
        subsets: string[]
    }

    interface FontFamily {
        family_name: string
        fonts: Font[]
        source?: "google"
        google?: GoogleMetadata
    }
}

export {}
