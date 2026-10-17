# CLAUDE.md — `flutter_tantivy` Instructions for Claude Code

This file provides guidance to **Claude Code (`claude`)** when working with the `flutter_tantivy` plugin or integrating `flutter_tantivy` into a Flutter application.

## 1. Activate the `flutter-tantivy` Skill

This project ships with a dedicated Agent Skill in both [`.claude/skills/flutter-tantivy/SKILL.md`](./.claude/skills/flutter-tantivy/SKILL.md) and [`skills/flutter-tantivy/SKILL.md`](./skills/flutter-tantivy/SKILL.md).

Whenever you are asked to implement search features, configure tokenizers, index documents, or modify `flutter_tantivy`:
1. Read [`skills/flutter-tantivy/SKILL.md`](./skills/flutter-tantivy/SKILL.md) for the step-by-step workflow and API selection guide.
2. Read [`skills/flutter-tantivy/references/api-reference.md`](./skills/flutter-tantivy/references/api-reference.md) for exact Dart FFI function signatures and model types (`Document`, `SearchResponse`, `LenientSearchResponse`, `SearchResult`, `TermSuggestion`, `TextRange`, `IndexSpaceUsage`).
3. Refer to [`skills/flutter-tantivy/examples/search_repository.dart`](./skills/flutter-tantivy/examples/search_repository.dart) for a complete, ready-to-use `LocalSearchRepository` implementation.

## 2. Core Usage Rules for `flutter_tantivy`

- **Mandatory Initialization**:
  1. `await RustLib.init();` (once before any FFI calls)
  2. `initTantivy(dirPath: dirPath, tokenizerType: 'cjk');`
- **Tokenizer Options (`tokenizerType`)**:
  - `'cjk'` or `'ngram'` for Korean/Japanese/Chinese + English mixed content
  - `'default'` for general Latin text
  - `'en_stem'` / `'stemmer'` / `'stem:<language>'` (18 languages, e.g. `'stem:english'`, `'stem:german'`) for morphological stemming
  - `'whitespace'` or `'raw'` for exact/whitespace splitting
- **Preferred Search APIs**:
  - Use `searchDocumentsLenient` for user-facing search inputs (tolerates malformed query syntax and returns `LenientSearchResponse` with `results` and non-fatal `errors` warnings).
  - Fall back to `searchDocumentsFuzzy` when results are empty to handle user typos.
  - Use `suggestTerms` for real-time prefix autocomplete suggestions from the FST term dictionary.
  - Use `searchSimilarDocuments` for "related/similar items" (`MoreLikeThisQuery`) recommendations.
  - Use `searchDocumentsRegex` and `searchDocumentsRegexPhrase` for regex and ordered regex sequence matching.
- **Highlighting Snippets & Byte Ranges**:
  - When `enableSnippet: true` is passed, `SearchResult` provides HTML-formatted `snippet` and `titleSnippet`, plus UTF-8 byte ranges in `fragmentRange` and `highlightedRanges` (`TextRange(start, end)`).

## 3. Plugin Development Commands

- **Regenerate FFI Bindings** (required after editing `rust/src/api/tantivy_api.rs`):
  ```bash
  flutter_rust_bridge_codegen generate
  ```
- **Run Static Analysis & Tests**:
  ```bash
  flutter analyze
  (cd example && flutter analyze)
  cargo test --manifest-path rust/Cargo.toml
  cargo clippy --manifest-path rust/Cargo.toml -- -D warnings
  ```
