# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- Comprehensive example app demonstrating all features
- Complete documentation and API reference

### Technical Details
- Built with Rust for native performance
- Integration via flutter_rust_bridge 2.11.1
- Tantivy search engine for fast indexing and querying
- FFI-based implementation for optimal performance
- Persistent storage with automatic index management

### Documentation
- Comprehensive README with quick start guide
- Example app with interactive UI
- API reference documentation
- Performance optimization tips
- Query syntax documentation

## [Unreleased]

### Planned Features
- Custom schema support for multiple field types
- Faceted search capabilities
- Fuzzy search support
- Highlighting of search results
- Index optimization and maintenance APIs
- Real-time search suggestions
- Multi-language tokenizer support
- Advanced query builder API

---

## Version History

### [0.1.0] - 2025-11-03
- Initial public release

---

## Migration Guide

### From Template
If you're migrating from the flutter_rust_bridge template:

1. Update your `pubspec.yaml`:
```yaml
dependencies:
  flutter_tantivy: ^0.1.0
```

2. Initialize the library:
```dart
await RustLib.init();
initTantivy(dirPath: yourIndexPath);
```

3. Replace template API calls with Tantivy API calls

---

## Breaking Changes

None yet - this is the initial release.

---

## Known Issues

None at this time.

For bug reports and feature requests, please visit:
https://github.com/yourusername/flutter_tantivy/issues

---

## Contributors

- Initial implementation and design
- Tantivy Rust library integration
- Flutter binding generation
- Example application development

Thank you to all contributors who made this project possible!

---

## Support

For questions and support:
- GitHub Issues: https://github.com/yourusername/flutter_tantivy/issues
- Documentation: See README.md
- Example: See example directory

---

*Note: This project uses [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge) for Dart-Rust interoperability and [Tantivy](https://github.com/quickwit-oss/tantivy) for search engine capabilities.*
