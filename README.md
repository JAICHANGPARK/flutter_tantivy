# flutter_tantivy

A Flutter plugin for full-text search powered by [Tantivy 0.26](https://github.com/quickwit-oss/tantivy), a fast full-text search engine library written in Rust. This plugin uses [flutter_rust_bridge 2.12](https://github.com/fzyzcjy/flutter_rust_bridge) to provide high-performance native search capabilities to Flutter applications.

[![pub package](https://img.shields.io/pub/v/flutter_tantivy.svg)](https://pub.dev/packages/flutter_tantivy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🚀 **High Performance**: Powered by Tantivy 0.26.1 Rust search engine with efficient indexing and searching
- 🔍 **Full-Text & Regex Search**: Support for standard text queries, boolean logic, wildcards, and regex pattern searches (`searchDocumentsRegex`)
- 🌏 **CJK Language Support**: Built-in N-gram tokenizer (`tokenizerType: 'cjk'`) for Chinese, Japanese, and Korean text tokenization and substring matching
- 📄 **Pagination & Total Hits**: Support for `offset` pagination and retrieving total matching document counts
- 💡 **Search Snippets & Highlighting**: Extract HTML-formatted highlighted snippets of matched search query terms
- 💾 **Persistent Storage**: Disk-based persistent index storage across app sessions
- 🔄 **CRUD & Bulk Operations**: Single & batch document creation, retrieval, updates, deletion, and index clearing
- 🔒 **Thread-Safe**: Safe concurrent access to the search index
- 📱 **Cross-Platform**: Supports Android, iOS, macOS, Linux, and Windows

## Installation

Add `flutter_tantivy` to your package's `pubspec.yaml` file:

```yaml
dependencies:
  flutter_tantivy: ^2026.7.26
```

Then run:

```bash
flutter pub get
```

## Quick Start

### 1. Initialize the Library

```dart
import 'package:flutter_tantivy/flutter_tantivy.dart';
import 'package:path_provider/path_provider.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize Rust FFI bridge
  await RustLib.init();

  // Initialize Tantivy index (pass tokenizerType: 'cjk' for Korean/Chinese/Japanese support)
  final directory = await getApplicationDocumentsDirectory();
  final indexPath = '${directory.path}/tantivy_index';
  initTantivy(dirPath: indexPath, tokenizerType: 'cjk');

  runApp(const MyApp());
}
```

### 2. Add Documents

```dart
// Add a single document with optional title
final doc = Document(
  id: '1',
  title: 'Flutter Framework',
  text: 'Flutter is an open-source UI toolkit by Google',
);
await addDocument(doc: doc);

// Batch add documents (more efficient)
final docs = [
  const Document(
    id: '1',
    title: 'Flutter UI',
    text: 'Flutter is a UI toolkit for building natively compiled apps',
  ),
  const Document(
    id: '2',
    title: 'Rust Language',
    text: 'Rust is a systems programming language focused on safety',
  ),
  const Document(
    id: '3',
    title: 'Tantivy Search',
    text: 'Tantivy is a full-text search engine library written in Rust',
  ),
];
await addDocumentsBatch(docs: docs);
```

### 3. Search Documents with Pagination & Snippets

```dart
final response = await searchDocuments(
  query: 'Flutter OR Rust',
  topK: BigInt.from(10),
  offset: BigInt.from(0),        // Optional: for pagination
  enableSnippet: true,            // Optional: generate highlighted text snippets
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
}
```

### 4. Regex Search

```dart
// Search using regular expressions across title and text
final regexResponse = await searchDocumentsRegex(
  pattern: 'Flutter.*',
  topK: BigInt.from(10),
);
print('Regex matches count: ${regexResponse.totalHits}');
```

### 5. Index Statistics, Cleaning & Release

```dart
// Get total document count in index
final count = getNumDocs();
print('Indexed Documents: $count');

// Delete all documents in index
await deleteAllDocuments();

// Close and release Tantivy resources (optional)
closeTantivy();
```

## API Reference

### Initialization & Info

- `initTantivy({required String dirPath})` - Initialize or open a Tantivy index at the specified directory
- `getNumDocs()` - Synchronously returns the total count of indexed documents
- `closeTantivy()` - Safely release and close Tantivy resources

### CRUD Operations

- `addDocument({required Document doc})` - Add or update a single document (auto-commits)
- `getDocumentById({required String id})` - Retrieve a document by its ID (synchronous)
- `updateDocument({required Document doc})` - Update an existing document
- `deleteDocument({required String id})` - Delete a document by ID
- `deleteAllDocuments()` - Delete all documents in the index

### Batch Operations

- `addDocumentsBatch({required List<Document> docs})` - Add multiple documents efficiently
- `deleteDocumentsBatch({required List<String> ids})` - Delete multiple documents efficiently

### Search Operations

- `searchDocuments({required String query, required BigInt topK, BigInt? offset, bool? enableSnippet})` - Search documents returning a `SearchResponse` containing total hits, matching results, and optional search snippets.

### Advanced Operations

- `addDocumentNoCommit({required Document doc})` - Add document without committing
- `deleteDocumentNoCommit({required String id})` - Delete document without committing
- `commit()` - Manually commit pending changes

### Data Types

#### Document
```dart
class Document {
  final String id;          // Unique identifier
  final String? title;      // Optional document title
  final String text;        // Searchable body text content

  const Document({required this.id, this.title, required this.text});
}
```

#### SearchResponse
```dart
class SearchResponse {
  final BigInt totalHits;            // Total matching documents count across all pages
  final List<SearchResult> results;  // Search results list for current page
}
```

#### SearchResult
```dart
class SearchResult {
  final double score;       // Relevance score (BM25)
  final Document doc;       // The matched document
  final String? snippet;    // HTML snippet highlighting search terms
}
```

## Query Syntax

Tantivy supports a rich query syntax matching across `title` and `text`:

- **Term search**: `flutter`
- **Phrase search**: `"flutter framework"`
- **Boolean operators**: `flutter AND dart`, `ios OR android`
- **Negation**: `flutter NOT web`
- **Field search**: `title:flutter` or `text:rust`
- **Wildcard**: `flut*`

## Platform-Specific Setup

### Android
Set minimum NDK version in `android/app/build.gradle`:
```gradle
android {
    ndkVersion "25.1.8937393" // or higher
}
```

### iOS / macOS
- Minimum iOS version: `11.0`
- Minimum macOS version: `10.11`

## Example App

Check out the [example](example) directory for an interactive Flutter application demonstrating all features.

## Architecture

This plugin uses:
- **Rust** for core search engine logic (Tantivy 0.26.1)
- **flutter_rust_bridge 2.12** for type-safe Dart-Rust FFI interop
- **Cargokit** for seamless multi-platform native build & bundling

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
