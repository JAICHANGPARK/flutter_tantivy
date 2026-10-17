---
name: flutter-tantivy
description: >-
  Integrates and implements high-performance full-text search in Flutter and Dart
  applications using the flutter_tantivy plugin (powered by Rust Tantivy 0.26.2).
  Use when adding local full-text search, CJK (Korean/Chinese/Japanese) N-gram search,
  18-language stemming, typo-tolerant fuzzy search, regex or regex-phrase search,
  More-Like-This similar document recommendations, prefix autocomplete suggestions,
  or highlighted search snippets in a Flutter app.
---

# Flutter Tantivy Skill (`flutter_tantivy`)

Use this skill when building, refactoring, or debugging local full-text search in a Flutter application with `package:flutter_tantivy`.

## 1. Setup & Initialization Workflow

1. **Add Dependencies** in `pubspec.yaml`:
   ```yaml
   dependencies:
     flutter_tantivy: ^2026.10.17
     path_provider: ^2.1.6
   ```

2. **Initialize FFI Bridge & Index Once at Startup**:
   Always call `await RustLib.init()` before any `flutter_tantivy` API call, then initialize the index directory with `initTantivy`:
   ```dart
   import 'package:flutter/widgets.dart';
   import 'package:flutter_tantivy/flutter_tantivy.dart';
   import 'package:path_provider/path_provider.dart';

   Future<void> initSearchEngine() async {
     WidgetsFlutterBinding.ensureInitialized();
     await RustLib.init();

     final dir = await getApplicationDocumentsDirectory();
     initTantivy(
       dirPath: '${dir.path}/tantivy_index',
       tokenizerType: 'cjk', // Choose based on language requirements below
     );
   }
   ```

3. **Choose the Appropriate `tokenizerType`**:
   - `'cjk'` or `'ngram'`: **Recommended for Korean, Chinese, Japanese, or mixed CJK + English apps.** Uses 1..3 character N-gram tokenization with lowercasing for substring matching.
   - `'default'`: Standard word tokenizer (`SimpleTokenizer` + 40-byte limit + `LowerCaser`). Best for space-delimited languages without stemming.
   - `'en_stem'` / `'stemmer'` / `'stem:<language>'`: Word tokenizer with language-specific stemming (`arabic`, `danish`, `dutch`, `english`, `finnish`, `french`, `german`, `greek`, `hungarian`, `italian`, `norwegian`, `portuguese`, `romanian`, `russian`, `spanish`, `swedish`, `tamil`, `turkish`).
   - `'whitespace'`: Splits only on whitespace and lowercases tokens.
   - `'raw'`: Emits the entire field value as a single token.

---

## 2. Choosing the Right Indexing Pattern

- **Single Document Auto-Commit (`addDocument` / `updateDocument` / `deleteDocument`)**:
  Use only for isolated single-item user actions. Each call acquires the index writer and commits immediately.
- **Bulk Operations (`addDocumentsBatch` / `deleteDocumentsBatch`)**:
  **Always prefer batch methods** when indexing or removing more than one document (e.g., initial sync, API page import). It performs a single Tantivy commit at the end.
- **Manual Transaction Control (`addDocumentNoCommit` / `deleteDocumentNoCommit` + `commit()`)**:
  Use when streaming documents across multiple steps before making them visible to searchers via a single synchronous `commit()`.

---

## 3. Choosing the Right Search API

| Use Case | Recommended Function | Notes |
| :--- | :--- | :--- |
| Standard query syntax (`AND`, `OR`, `NOT`, `"phrase"~2`, `"prefix"*`, `title:foo^2`, `title:*`, `/regex/`) | `searchDocuments` | Throws on invalid query syntax. |
| Direct end-user search bar input | `searchDocumentsLenient` | Never fails on malformed syntax (e.g. unclosed quotes); returns `LenientSearchResponse` with `results` + `errors` warnings, and supports `conjunctionByDefault` and `titleBoost`. |
| Typo-tolerant search | `searchDocumentsFuzzy` | Levenshtein automaton (`distance`: `1` or `2`, `transpositionCostOne`: `true`, `prefix`: `true`/`false`). Works on both word and CJK indexes. |
| Single regular expression | `searchDocumentsRegex` | Matches regex pattern across `title` and `text`. |
| Ordered regex sequence with proximity | `searchDocumentsRegexPhrase` | Matches `patterns` in order within `slop` token distance. |
| "Related / Similar items" recommendation | `searchSimilarDocuments` | Uses Tantivy's `MoreLikeThisQuery` for a given document `id` (automatically excludes the source document itself). |
| Filter documents where a field is non-null | `searchDocumentsExists` | Fast-field columnar check (`fieldName`: `'title'`, `'text'`, or `'id'`). |
| Search bar autocomplete / typeahead | `suggestTerms` | Streams matching terms and `docFreq` from the FST term dictionary by `prefix`. |

---

## 4. Highlighting Snippets & Byte Ranges

When `enableSnippet: true` is passed to search functions, each `SearchResult` includes:
- `snippet`: HTML-highlighted body snippet (e.g. `<b>Flutter</b> is a UI...`).
- `titleSnippet`: HTML-highlighted title snippet if the query matched `title`.
- `fragmentRange`: `TextRange(start, end)` byte range of the extracted snippet fragment within `doc.text`.
- `highlightedRanges`: `List<TextRange>` byte ranges of matched terms inside the snippet fragment.

---

## 5. Additional References & Templates

- For full function signatures and data classes, read [references/api-reference.md](./references/api-reference.md).
- For a production-ready repository wrapper pattern, read [examples/search_repository.dart](./examples/search_repository.dart).
