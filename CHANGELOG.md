# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [2026.7.26] - 2026-07-26

### Added
- **Regex Search Support (`searchDocumentsRegex`)**: Added support for searching documents using regular expression patterns (`RegexQuery`). (Addresses Issue #1)
- **CJK / Multi-language Tokenizer (`tokenizerType: 'cjk'`)**: Added N-gram tokenizer option during `initTantivy` for Chinese, Japanese, and Korean (CJK) text tokenization and substring search. (Addresses Issues #2, #3, #4)
- **Search Snippets & Highlighting (`snippet`)**: Added search term highlighting using Tantivy's `SnippetGenerator`. Returns HTML-formatted snippets (e.g. `<b>Flutter</b> is a UI...`).
- **Pagination Support (`offset`)**: Added `offset` parameter to `searchDocuments` for paginated search results.
- **Total Hits Count (`totalHits`)**: `searchDocuments` now returns a `SearchResponse` containing total matching documents count (`totalHits`) and page results.
- **Document Title Support (`title`)**: Added optional `title` field to `Document`. Searching matches across both `title` and `text` fields.
- **Get Total Document Count (`getNumDocs()`)**: Added synchronous function to retrieve total number of indexed documents.
- **Delete All Documents (`deleteAllDocuments()`)**: Added function to clear all documents from the search index.
- **Close Tantivy (`closeTantivy()`)**: Added function to safely close and reset Tantivy resources.

### Changed
- **Date-Based Versioning (CalVer)**: Switched package versioning strategy to date-based versioning (`2026.7.26`).
- **Tantivy Upgrade**: Upgraded Tantivy engine from `0.25.0` to **`0.26.1`**.
- **flutter_rust_bridge Upgrade**: Upgraded `flutter_rust_bridge` from `2.11.1` to **`2.12.0`**.

---

## [0.1.0] - 2025-11-03

### Added
- Initial release of flutter_tantivy plugin
- Full-text search powered by Tantivy search engine
- Complete CRUD operations for document management
  - `addDocument()` - Add single document with auto-commit
  - `getDocumentById()` - Retrieve document by ID (synchronous)
  - `updateDocument()` - Update existing document
  - `deleteDocument()` - Delete document by ID
- Batch operations for improved performance
  - `addDocumentsBatch()` - Add multiple documents efficiently
  - `deleteDocumentsBatch()` - Delete multiple documents efficiently
- Advanced transaction control
  - `addDocumentNoCommit()` - Add document without committing
  - `deleteDocumentNoCommit()` - Delete document without committing
  - `commit()` - Manually commit pending changes
- Search functionality with relevance scoring
  - `searchDocuments()` - Full-text search with query parsing
  - Support for boolean operators (AND, OR, NOT)
  - Phrase search and wildcard support
- Index management
  - `initTantivy()` - Initialize or open persistent search index
  - Automatic index reload on commits
  - Thread-safe concurrent access
- Cross-platform support
  - Android (API 21+)
  - iOS (11.0+)
  - macOS (10.11+)
  - Linux
  - Windows
