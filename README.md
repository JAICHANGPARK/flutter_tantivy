# flutter_tantivy

A Flutter plugin for full-text search powered by [Tantivy](https://github.com/quickwit-oss/tantivy), a fast full-text search engine library written in Rust. This plugin uses [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge) to provide high-performance native search capabilities to Flutter applications.

[![pub package](https://img.shields.io/pub/v/flutter_tantivy.svg)](https://pub.dev/packages/flutter_tantivy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🚀 **High Performance**: Native Rust implementation with efficient indexing and searching
- 🔍 **Full-Text Search**: Advanced search capabilities with query parsing
- 💾 **Persistent Storage**: Index data persists across app sessions
- 🔄 **CRUD Operations**: Complete create, read, update, and delete operations
- 📦 **Batch Operations**: Efficient batch insertions and deletions
- 🎯 **Relevance Scoring**: Search results ordered by relevance score
- 🔒 **Thread-Safe**: Safe concurrent access to the search index
- 📱 **Cross-Platform**: Supports Android, iOS, macOS, Linux, and Windows

## Installation

Add this to your package's `pubspec.yaml` file:

```yaml
dependencies:
  flutter_tantivy: ^0.1.0
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

  // Initialize Rust library
  await RustLib.init();

  // Initialize Tantivy index
  final directory = await getApplicationDocumentsDirectory();
  final indexPath = '${directory.path}/tantivy_index';
  initTantivy(dirPath: indexPath);

  runApp(MyApp());
}
```

### 2. Add Documents

```dart
// Add a single document
final doc = Document(
  id: '1',
  text: 'Flutter is an open-source UI toolkit by Google',
);
await addDocument(doc: doc);

// Batch add documents (more efficient)
final docs = [
  Document(id: '1', text: 'Flutter is a UI toolkit'),
  Document(id: '2', text: 'Rust is a systems programming language'),
  Document(id: '3', text: 'Tantivy is a search engine library'),
];
await addDocumentsBatch(docs: docs);
```

### 3. Search Documents

```dart
final results = await searchDocuments(
  query: 'Flutter OR Rust',
  topK: BigInt.from(10),
);

for (final result in results) {
  print('Score: ${result.score}');
  print('ID: ${result.doc.id}');
  print('Text: ${result.doc.text}');
}
```

### 4. Get Document by ID

```dart
final doc = getDocumentById(id: '1');
if (doc != null) {
  print('Found: ${doc.text}');
}
```

### 5. Update Document

```dart
final updatedDoc = Document(
  id: '1',
  text: 'Flutter is an open-source UI toolkit created by Google',
);
await updateDocument(doc: updatedDoc);
```

### 6. Delete Document

```dart
// Delete single document
await deleteDocument(id: '1');

// Batch delete documents
await deleteDocumentsBatch(ids: ['1', '2', '3']);
```

## Advanced Usage

### Manual Transaction Control

For advanced users who need fine-grained control over commits:

```dart
// Add documents without committing
await addDocumentNoCommit(doc: doc1);
await addDocumentNoCommit(doc: doc2);
await deleteDocumentNoCommit(id: 'old_id');

// Manually commit all changes
commit();
```

This is useful when you need to perform multiple operations atomically.

## API Reference

### Initialization

- `initTantivy({required String dirPath})` - Initialize or open a Tantivy index at the specified directory

### CRUD Operations

- `addDocument({required Document doc})` - Add a single document (auto-commits)
- `getDocumentById({required String id})` - Retrieve a document by its ID (synchronous)
- `updateDocument({required Document doc})` - Update an existing document
- `deleteDocument({required String id})` - Delete a document by ID

### Batch Operations

- `addDocumentsBatch({required List<Document> docs})` - Add multiple documents efficiently
- `deleteDocumentsBatch({required List<String> ids})` - Delete multiple documents efficiently

### Search Operations

- `searchDocuments({required String query, required BigInt topK})` - Search documents with a query string

### Advanced Operations

- `addDocumentNoCommit({required Document doc})` - Add document without committing
- `deleteDocumentNoCommit({required String id})` - Delete document without committing
- `commit()` - Manually commit pending changes

### Data Types

#### Document
```dart
class Document {
  final String id;    // Unique identifier
  final String text;  // Searchable text content
}
```

#### SearchResult
```dart
class SearchResult {
  final double score;      // Relevance score
  final Document doc;      // The matched document
}
```

## Query Syntax

Tantivy supports a rich query syntax:

- **Term search**: `flutter`
- **Phrase search**: `"flutter framework"`
- **Boolean operators**: `flutter AND dart`, `ios OR android`
- **Negation**: `flutter NOT web`
- **Field search**: `text:flutter` (when using custom schemas)
- **Wildcard**: `flut*`

## Performance Tips

1. **Use Batch Operations**: When adding or deleting multiple documents, use `addDocumentsBatch` and `deleteDocumentsBatch` instead of individual operations
2. **Manual Commits**: For bulk operations, use `addDocumentNoCommit` and call `commit()` once at the end
3. **Index Location**: Store the index on local storage, not in temporary directories
4. **Query Optimization**: Keep queries simple and specific for better performance

## Platform-Specific Setup

### Android

Add the following to your `android/app/build.gradle`:

```gradle
android {
    ndkVersion "25.1.8937393"  // or higher
}
```

### iOS

Minimum iOS version: 11.0

### macOS

Minimum macOS version: 10.11

## Example

Check out the [example](example) directory for a complete working demo app that demonstrates all the features of this plugin.

## Architecture

This plugin uses:
- **Rust** for the core search engine implementation (Tantivy)
- **flutter_rust_bridge** for seamless Dart-Rust interop
- **FFI** for native performance

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tantivy](https://github.com/quickwit-oss/tantivy) - The amazing full-text search engine
- [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge) - For making Rust-Flutter integration seamless

## Support

If you find this package useful, please consider giving it a ⭐ on [GitHub](https://github.com/yourusername/flutter_tantivy)!

For bugs and feature requests, please file an issue on the [GitHub issue tracker](https://github.com/yourusername/flutter_tantivy/issues).
