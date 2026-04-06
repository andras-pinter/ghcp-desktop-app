/** Types for web search results and URL content extraction. */

/** A web search result from the Bing Web Search API. */
export interface SearchResult {
  title: string;
  url: string;
  snippet: string;
  displayUrl?: string | null;
}

/** Extracted readable content from a fetched URL. */
export interface ExtractedContent {
  title: string | null;
  text: string;
  url: string;
  byteLength: number;
  truncated: boolean;
}

/** Preview info for a URL attached in the input area. */
export interface UrlPreview {
  /** The original URL. */
  url: string;
  /** Domain extracted from the URL (e.g. "docs.rs"). */
  domain: string;
  /** Extracted content (populated after fetch completes). */
  content: ExtractedContent | null;
  /** Whether the fetch is in progress. */
  loading: boolean;
  /** Error message if fetch failed. */
  error: string | null;
}
