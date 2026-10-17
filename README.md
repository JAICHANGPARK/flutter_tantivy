# flutter_tantivy

A Flutter plugin for full-text search powered by [Tantivy 0.26.2](https://github.com/quickwit-oss/tantivy), a fast full-text search engine library written in Rust. This plugin uses [flutter_rust_bridge 2.13](https://github.com/fzyzcjy/flutter_rust_bridge) to provide high-performance native search capabilities to Flutter applications.

[![pub package](https://img.shields.io/pub/v/flutter_tantivy.svg)](https://pub.dev/packages/flutter_tantivy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🚀 **High Performance**: Powered by Tantivy 0.26.2 Rust search engine with fast columnar storage and inverted indexing
- 🔍 **Full-Text, Regex & Regex Phrase Search**: Support for standard text queries, boolean logic, wildcards, inline `/regex/`, `searchDocumentsRegex`, and proximity-based `searchDocumentsRegexPhrase`
- 🔤 **Typo-Tolerant Fuzzy Search**: Levenshtein automaton fuzzy search (`searchDocumentsFuzzy`) with configurable edit distance, transposition cost, and prefix matching
- 🔗 **More-Like-This (Similar Documents)**: Find similar documents (`searchSimilarDocuments`) based on document term vectors and frequencies
- 🛡️ **Lenient Query Parsing & `field:*` Exists Syntax**: Fault-tolerant query parsing (`searchDocumentsLenient`), field boost, default AND mode, and `field:*` / `searchDocumentsExists` support
- 💬 **Term Dictionary Autocomplete (`suggestTerms`)**: Stream terms and document frequencies by prefix from Tantivy's FST term dictionary
- 🌏 **CJK & 18-Language Stemmer Support**: Built-in N-gram tokenizer (`tokenizerType: 'cjk'`), whitespace/raw tokenizers, and 18-language stemming (`tokenizerType: 'en_stem'` or `'stem:<language>'`)
- 💡 **Rich Snippets & Byte Ranges**: Extract HTML-formatted highlighted snippets (`snippet`, `titleSnippet`) along with fragment byte ranges (`fragmentRange`) and highlight offsets (`highlightedRanges`)
- 📊 **Index Space Usage & Segment Inspection**: Inspect per-component storage metrics (`getIndexSpaceUsage`) and active segment files (`listIndexSegmentFiles`)
- 🤖 **AI Agent Skill Ready**: Bundled `skills/flutter-tantivy` skill package for AI coding assistants (Antigravity, Gemini CLI, Claude Code, Cursor)
- 📱 **Cross-Platform**: Supports Android, iOS, macOS, Linux, and Windows

## Installation

Add `flutter_tantivy` to your package's `pubspec.yaml` file:

```yaml
dependencies:
  flutter_tantivy: ^2026.10.17
```

Then run:

```bash
flutter pub get
```

---

## 🤖 AI Agent Skills Guide (AI 에이전트 Skills 활용 가이드)

This repository includes a dedicated AI agent skill in [`skills/flutter-tantivy/`](skills/flutter-tantivy) so AI coding assistants can accurately set up, index, query, and optimize `flutter_tantivy` without hallucinating APIs.

### Skill Directory Structure

```text
skills/flutter-tantivy/
├── SKILL.md                        # Core workflow, tokenizer selection & search API decision matrix
├── references/
│   └── api-reference.md            # Complete Dart function signatures & data models
└── examples/
    └── search_repository.dart      # Production-ready repository pattern (lenient + fuzzy fallback)
```

### How to Install & Use the Skill

#### 1. Using `npx skills` CLI (Universal)
```bash
npx skills add JAICHANGPARK/flutter_tantivy
```

#### 2. Google Antigravity (AGY) / Gemini CLI
Copy the skill into your project's `.agents/skills/` directory or your global `~/.gemini/config/skills/` directory:
```bash
# Project-scoped (recommended for team sharing)
mkdir -p .agents/skills
cp -R path/to/flutter_tantivy/skills/flutter-tantivy .agents/skills/

# Or global configuration
mkdir -p ~/.gemini/config/skills
cp -R path/to/flutter_tantivy/skills/flutter-tantivy ~/.gemini/config/skills/
```

#### 3. Claude Code / Cursor
Copy the skill folder to `.claude/skills/` or `.cursor/skills/` in your workspace:
```bash
mkdir -p .claude/skills
cp -R path/to/flutter_tantivy/skills/flutter-tantivy .claude/skills/
```

#### 4. Example Prompts with the Skill Active
- *"Use `flutter-tantivy` to create a local Korean/English search service with typo-tolerant fuzzy fallback and search term highlighting."*
- *"Add prefix autocomplete suggestions and 'related articles' (`searchSimilarDocuments`) to my detail screen using `flutter_tantivy`."*

---

## Quick Start

### 1. Initialize the Library

```dart
import 'package:flutter_tantivy/flutter_tantivy.dart';
import 'package:path_provider/path_provider.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize Rust FFI bridge
  await RustLib.init();

  // Initialize Tantivy index
  // Supported tokenizerType values:
  // - 'default': Standard SimpleTokenizer + LowerCaser
  // - 'cjk' / 'ngram': N-gram (1..3) tokenizer for Korean/Chinese/Japanese
  // - 'en_stem' / 'stem:<language>': 18-language stemmer (english, french, german, spanish, etc.)
  // - 'whitespace' / 'raw'
  final directory = await getApplicationDocumentsDirectory();
  final indexPath = '${directory.path}/tantivy_index';
  initTantivy(dirPath: indexPath, tokenizerType: 'cjk');

  runApp(const MyApp());
}
```

### 2. Add Documents

```dart
// Add a single document with optional title
const doc = Document(
  id: '1',
  title: 'Flutter Framework',
  text: 'Flutter is an open-source UI toolkit by Google',
);
await addDocument(doc: doc);

// Batch add documents (more efficient)
const docs = [
  Document(
    id: '1',
    title: 'Flutter UI',
    text: 'Flutter is a UI toolkit for building natively compiled apps',
  ),
  Document(
    id: '2',
    title: 'Rust Language',
    text: 'Rust is a systems programming language focused on safety',
  ),
  Document(
    id: '3',
    title: 'Tantivy Search',
    text: 'Tantivy is a full-text search engine library written in Rust',
  ),
];
await addDocumentsBatch(docs: docs);
```

### 3. Search Documents with Pagination, Snippets & Fragment Byte Ranges

```dart
final response = await searchDocuments(
  query: 'Flutter OR Rust',
  topK: BigInt.from(10),
  offset: BigInt.from(0),        // Optional: for pagination
  enableSnippet: true,            // Optional: generate highlighted text snippets & byte ranges
);

print('Total matching documents: ${response.totalHits}');

for (final result in response.results) {
  print('Score: ${result.score}');
  print('ID: ${result.doc.id}');
  print('Title: ${result.doc.title}');
  print('Text: ${result.doc.text}');
  if (result.snippet != null) {
    print('Snippet: ${result.snippet}'); // e.g. "<b>Flutter</b> is a UI..."
  }
  if (result.fragmentRange != null) {
    print('Fragment byte range: ${result.fragmentRange!.start}..${result.fragmentRange!.end}');
  }
}
```

### 4. Fuzzy Search, Regex Phrase & Similar Documents (MoreLikeThis)

```dart
// Typo-tolerant Fuzzy Search (Levenshtein distance 1..2)
final fuzzyResponse = await searchDocumentsFuzzy(
  term: 'Fluter',
  distance: 1,
  transpositionCostOne: true,
  prefix: true,
  topK: BigInt.from(10),
  enableSnippet: true,
);

// Regex Phrase Search with proximity slop
final phraseResponse = await searchDocumentsRegexPhrase(
  patterns: ['flut.*', 'toolkit'],
  slop: 2,
  topK: BigInt.from(10),
);

// Find Similar Documents (MoreLikeThisQuery)
final similarResponse = await searchSimilarDocuments(
  id: '1',
  topK: BigInt.from(5),
);

// Term Dictionary Prefix Autocomplete
final suggestions = await suggestTerms(
  prefix: 'flut',
  limit: BigInt.from(5),
);
```

### 5. Index Statistics, Space Usage & Release

```dart
// Get total document count in index
final count = getNumDocs();
print('Indexed Documents: $count');

// Inspect detailed index storage space usage
final usage = getIndexSpaceUsage();
print('Total bytes: ${usage.totalBytes}, Segments: ${usage.numSegments}');

// List active segment files in index metadata
final segmentFiles = listIndexSegmentFiles();
print('Segment files: $segmentFiles');

// Delete all documents in index
await deleteAllDocuments();

// Close and release Tantivy resources (optional)
closeTantivy();
```

## API Reference

### Initialization & Diagnostics

- `initTantivy({required String dirPath, String? tokenizerType})` - Initialize or open a Tantivy index at the specified directory
- `getNumDocs()` - Synchronously returns the total count of indexed documents
- `getIndexSpaceUsage()` - Synchronously returns `IndexSpaceUsage` with byte breakdown across store, postings, positions, fast fields, termdict, fieldnorms, and deletes
- `listIndexSegmentFiles()` - Synchronously returns candidate relative paths for all active segments in `IndexMeta`
- `closeTantivy()` - Safely release and close Tantivy resources

### CRUD & Batch Operations

- `addDocument({required Document doc})` - Add or update a single document (auto-commits)
- `getDocumentById({required String id})` - Retrieve a document by its ID (synchronous)
- `updateDocument({required Document doc})` - Update an existing document
- `deleteDocument({required String id})` - Delete a document by ID
- `deleteAllDocuments()` - Delete all documents in the index
- `addDocumentsBatch({required List<Document> docs})` - Add multiple documents efficiently
- `deleteDocumentsBatch({required List<String> ids})` - Delete multiple documents efficiently
- `addDocumentNoCommit({required Document doc})` - Add document without committing
- `deleteDocumentNoCommit({required String id})` - Delete document without committing
- `commit()` - Manually commit pending changes

### Search & Suggestion Operations

- `searchDocuments({required String query, required BigInt topK, BigInt? offset, bool? enableSnippet})` - Full-text search supporting boolean operators, phrase slop (`"..."~N`), phrase prefix (`"..."*`), boost (`^N`), inline regex (`/pattern/`), and field existence (`title:*`)
- `searchDocumentsLenient({required String query, required BigInt topK, BigInt? offset, bool? enableSnippet, bool? conjunctionByDefault, double? titleBoost})` - Lenient search returning `LenientSearchResponse` (results + non-fatal syntax errors)
- `searchDocumentsRegex({required String pattern, required BigInt topK, BigInt? offset, bool? enableSnippet})` - Regex pattern search across `title` and `text`
- `searchDocumentsRegexPhrase({required List<String> patterns, int? slop, int? maxExpansions, required BigInt topK, BigInt? offset, bool? enableSnippet})` - Ordered sequence of regex patterns with optional `slop`
- `searchDocumentsFuzzy({required String term, int? distance, bool? transpositionCostOne, bool? prefix, required BigInt topK, BigInt? offset, bool? enableSnippet})` - Typo-tolerant Levenshtein automaton search
- `searchSimilarDocuments({required String id, required BigInt topK, ...})` - `MoreLikeThisQuery` similarity search for a reference document ID
- `searchDocumentsExists({required String fieldName, required BigInt topK, BigInt? offset})` - Match documents where `fieldName` (`id`, `title`, or `text`) has a non-null value
- `suggestTerms({required String prefix, BigInt? limit, bool? includeTitle})` - Stream prefix suggestions and document frequencies from the term dictionary

## Query Syntax

Tantivy supports a rich query syntax matching across `title` and `text`:

- **Term search**: `flutter`
- **Phrase & Slop search**: `"flutter framework"`, `"flutter toolkit"~2`
- **Phrase prefix search**: `"flutter frame"*`
- **Boolean operators**: `flutter AND dart`, `ios OR android`, `flutter NOT web`
- **Field search & Boost**: `title:flutter^2.0` or `text:rust`
- **Exists query**: `title:*`, `* NOT title:*`
- **Inline Regex & Wildcard**: `title:/flut.*/`, `flut*`

## Architecture

This plugin uses:
- **Rust** for core search engine logic (Tantivy 0.26.2)
- **flutter_rust_bridge 2.13** for type-safe Dart-Rust FFI interop
- **Cargokit** for seamless multi-platform native build & bundling

## 🤖 AI Agent Skills Guide (Claude, Codex, Gemini / Antigravity 활용 가이드)

`flutter_tantivy` includes built-in **AI Agent Skills** and instruction files so AI coding assistants (**Claude Code**, **OpenAI Codex**, and **Google Antigravity / Gemini CLI**) can automatically understand the FFI initialization flow, tokenizer options (`cjk`, `default`, `en_stem`, `stem:<lang>`, `whitespace`, `raw`), and search APIs without hallucinating outdated bindings.

### 📁 Included Skill & Agent Files

```text
flutter_tantivy/
├── skills/flutter-tantivy/                 # Canonical Agent Skill package
│   ├── SKILL.md                            # Step-by-step workflow & API selection table
│   ├── references/api-reference.md         # Complete Dart FFI signatures & data models
│   └── examples/search_repository.dart     # Ready-to-use LocalSearchRepository template
├── .agents/skills/flutter-tantivy/         # Auto-discovered by Gemini (Antigravity) & Codex
├── .claude/skills/flutter-tantivy/         # Auto-discovered by Claude Code
├── .codex/skills/flutter-tantivy/          # Auto-discovered by OpenAI Codex
├── CLAUDE.md                               # Root context for Claude Code
├── AGENTS.md                               # Root context for OpenAI Codex & Antigravity
└── GEMINI.md                               # Root context for Gemini CLI & Antigravity
```

---

### 🚀 How to Use in Your Flutter Project (내 Flutter 앱 프로젝트에 적용하기)

#### 1. Quick Install via `npx skills` CLI (권장)

Install the `flutter-tantivy` skill directly into your Flutter app workspace for **Claude Code**, **Codex**, and **Gemini (Antigravity)**:

```bash
# Install for all detected agents (Claude, Codex, Antigravity/Gemini)
npx skills add jaichangpark/flutter_tantivy

# Or target a specific agent:
npx skills add jaichangpark/flutter_tantivy -a claude-code
npx skills add jaichangpark/flutter_tantivy -a codex
npx skills add jaichangpark/flutter_tantivy -a antigravity
```

---

#### 2. Agent-Specific Setup (에이전트별 수동 설정 방법)

##### 🟣 Claude Code (`claude`)
Claude Code automatically loads skills from `.claude/skills/<skill-name>/SKILL.md` and project instructions from `CLAUDE.md`.

```bash
# Copy skill into your Flutter project's .claude/skills directory
mkdir -p .claude/skills
cp -R /path/to/flutter_tantivy/skills/flutter-tantivy .claude/skills/flutter-tantivy
```
- **Usage in Claude Code**: Simply ask in natural language or invoke `/flutter-tantivy`:
  > *"Use the `flutter-tantivy` skill to implement Korean+English full-text search with fuzzy fallback and highlighted snippets."*

##### 🟢 OpenAI Codex (`codex`)
OpenAI Codex CLI automatically reads `AGENTS.md` and loads workspace skills from `.agents/skills/<skill-name>/SKILL.md` (or `.codex/skills/<skill-name>/SKILL.md`).

```bash
# Copy skill into your Flutter project's .agents/skills directory
mkdir -p .agents/skills
cp -R /path/to/flutter_tantivy/skills/flutter-tantivy .agents/skills/flutter-tantivy
```
- Optionally add a reference in your project's `AGENTS.md`:
  ```markdown
  - For local full-text search using `flutter_tantivy`, read `.agents/skills/flutter-tantivy/SKILL.md`.
  ```

##### 🔵 Gemini CLI / Google Antigravity (`gemini` / `agy`)
Google Antigravity and Gemini CLI automatically discover workspace skills in `.agents/skills/<skill-name>/SKILL.md`, global skills in `~/.gemini/config/skills/<skill-name>/SKILL.md`, and rules in `GEMINI.md` / `AGENTS.md`.

```bash
# Option A: Workspace-level (recommended for team projects via Git)
mkdir -p .agents/skills
cp -R /path/to/flutter_tantivy/skills/flutter-tantivy .agents/skills/flutter-tantivy

# Option B: Global machine-level (available across all your Flutter projects)
mkdir -p ~/.gemini/config/skills
cp -R /path/to/flutter_tantivy/skills/flutter-tantivy ~/.gemini/config/skills/flutter-tantivy
```

---

### 💡 Example Prompts for AI Agents (프롬프트 활용 예시)

Once the skill is loaded in **Claude**, **Codex**, or **Gemini (Antigravity)**, try asking:

1. **초기화 및 한국어/영어 통합 검색 서비스 구현**:
   > "`flutter-tantivy` 스킬을 참고해서 `tokenizerType: 'cjk'`로 인덱스를 초기화하고, 사용자 검색어 입력 시 `searchDocumentsLenient`로 검색하되 결과가 없으면 `searchDocumentsFuzzy`로 오타 보정 검색을 수행하는 `SearchService`를 작성해 줘."
2. **실시간 자동완성 및 접두어 추천 검색 UI 구현**:
   > "`flutter-tantivy` 스킬의 `suggestTerms`와 `searchDocumentsLenient`를 활용해서 검색창 입력 중 실시간 추천 검색어 칩(Chip)과 문서 빈도(`docFreq`)를 보여주는 Flutter 위젯을 만들어 줘."
3. **검색 결과 스니펫 및 바이트 오프셋 하이라이팅 구현**:
   > "`flutter-tantivy` 스킬의 `SearchResult.snippet`, `titleSnippet`, `highlightedRanges`(`TextRange`)를 사용해서 검색어 매칭 구간을 하이라이팅하는 리스트 아이템 위젯을 만들어 줘."
4. **연관 게시글(MoreLikeThis) 추천 기능 추가**:
   > "현재 보고 있는 문서 ID를 기준으로 `searchSimilarDocuments`를 호출해 하단에 '유사한 글 5개'를 추천하는 기능을 추가해 줘."

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

