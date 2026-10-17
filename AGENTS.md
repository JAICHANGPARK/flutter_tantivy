# AGENTS.md — `flutter_tantivy` Guide for AI Coding Agents (Codex, Gemini, Antigravity, Claude)

This repository provides **`flutter_tantivy`**, a high-performance full-text search plugin for Flutter powered by Rust [Tantivy 0.26.2](https://github.com/quickwit-oss/tantivy) and `flutter_rust_bridge`.

## 1. AI Agent Skill (`flutter-tantivy`)

Whenever a task involves integrating, using, extending, or debugging `flutter_tantivy` (such as initializing the Tantivy index, selecting tokenizers, indexing documents, or implementing BM25/lenient/fuzzy/regex/regex-phrase/autocomplete/MoreLikeThis search), **read and follow the `flutter-tantivy` skill first**:

- **Skill Instructions**: [`skills/flutter-tantivy/SKILL.md`](./skills/flutter-tantivy/SKILL.md)
  - Auto-discovered via `.agents/skills/flutter-tantivy/SKILL.md`, `.codex/skills/flutter-tantivy/SKILL.md`, and `.claude/skills/flutter-tantivy/SKILL.md`
- **Full Dart FFI API Reference**: [`skills/flutter-tantivy/references/api-reference.md`](./skills/flutter-tantivy/references/api-reference.md)
- **Production Repository Pattern**: [`skills/flutter-tantivy/examples/search_repository.dart`](./skills/flutter-tantivy/examples/search_repository.dart)

## 2. Quick API & Tokenizer Rules

1. **Initialization Sequence**:
   - Always call `await RustLib.init();` once at startup before any FFI calls.
   - Then call `initTantivy(dirPath: dirPath, tokenizerType: tokenizerType);`.
2. **Tokenizer Selection (`tokenizerType`)**:
   - `'cjk'` or `'ngram'`: Korean / Japanese / Chinese / Mixed text (1..3 character N-gram + lowercasing).
   - `'default'`: General English/Latin text (splits on punctuation, lowercases, removes tokens > 40 bytes).
   - `'en_stem'` / `'stemmer'` / `'stem:<language>'`: Language stemming for 18 languages (e.g., `'stem:english'`, `'stem:french'`, `'stem:spanish'`, `'stem:german'`).
   - `'whitespace'`: Splits purely on whitespace and lowercases.
   - `'raw'`: Exact untokenized match.
3. **Choosing the Right Search Function**:
   - User-typed search bar (recommended default): `searchDocumentsLenient(...)` (handles unclosed quotes/parentheses without throwing, returns `LenientSearchResponse` with `results` + `errors`).
   - Standard query syntax (`AND`, `OR`, `NOT`, `"phrase"~2`, `"prefix"*`, `title:*`, `/regex/`): `searchDocuments(...)`.
   - Typo-tolerant fallback: `searchDocumentsFuzzy(..., distance: 1, transpositionCostOne: true, prefix: true)`.
   - Regex & Regex Phrase search: `searchDocumentsRegex(...)` and `searchDocumentsRegexPhrase(patterns: [...], slop: 1, ...)`.
   - Autocomplete / prefix suggestions: `suggestTerms(prefix: ..., limit: BigInt.from(10), includeTitle: true)`.
   - Similar documents ("Related posts"): `searchSimilarDocuments(id: ..., topK: BigInt.from(5))`.
   - Field existence filter: `searchDocumentsExists(fieldName: 'title', topK: BigInt.from(20))`.

## 3. Repository Structure & Development Workflow

- `lib/`: Public Dart API (`lib/flutter_tantivy.dart`) and generated FFI bindings (`lib/src/rust/api/tantivy_api.dart`).
- `rust/`: Rust crate (`rust/src/api/tantivy_api.rs`) wrapping Tantivy `0.26.2`.
- `example/`: Sample Flutter application demonstrating all search modes.

### Code Generation (`flutter_rust_bridge`)
If you modify `rust/src/api/tantivy_api.rs`, regenerate the Dart/Rust FFI glue code:
```bash
flutter_rust_bridge_codegen generate
```

### Verification Commands
```bash
# 1. Dart/Flutter static analysis
flutter analyze
(cd example && flutter analyze)

# 2. Rust unit tests & linter
cargo test --manifest-path rust/Cargo.toml
cargo clippy --manifest-path rust/Cargo.toml -- -D warnings
```
