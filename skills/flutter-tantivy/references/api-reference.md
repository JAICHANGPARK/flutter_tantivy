# `flutter_tantivy` Complete API Reference

Import the public entrypoint:
```dart
import 'package:flutter_tantivy/flutter_tantivy.dart';
```

## Lifecycle & Diagnostics

```dart
// Initialize FFI runtime once before calling any plugin functions
await RustLib.init();

// Initialize or open an index at dirPath
void initTantivy({required String dirPath, String? tokenizerType});

// Release index reader/writer locks and close Tantivy state
void closeTantivy();

// Total number of indexed documents
BigInt getNumDocs();

// Detailed storage space breakdown across segments
IndexSpaceUsage getIndexSpaceUsage();

// Candidate relative file paths for all active segments in IndexMeta
List<String> listIndexSegmentFiles();
```

## CRUD & Transaction Operations

```dart
// Single document operations (auto-commit)
Future<void> addDocument({required Document doc});
Future<void> updateDocument({required Document doc});
Future<void> deleteDocument({required String id});
Document? getDocumentById({required String id});

// Bulk operations (single commit)
Future<void> addDocumentsBatch({required List<Document> docs});
Future<void> deleteDocumentsBatch({required List<String> ids});
Future<void> deleteAllDocuments();

// Manual commit operations
Future<void> addDocumentNoCommit({required Document doc});
Future<void> deleteDocumentNoCommit({required String id});
void commit();
```

## Search & Autocomplete Operations

```dart
// 1. Standard Query Parser Search
Future<SearchResponse> searchDocuments({
  required String query,
  required BigInt topK,
  BigInt? offset,
  bool? enableSnippet,
});

// 2. Lenient Query Parser Search (safe for raw user input)
Future<LenientSearchResponse> searchDocumentsLenient({
  required String query,
  required BigInt topK,
  BigInt? offset,
  bool? enableSnippet,
  bool? conjunctionByDefault,
  double? titleBoost,
});

// 3. Typo-Tolerant Fuzzy Search (Levenshtein distance 0..=2)
Future<SearchResponse> searchDocumentsFuzzy({
  required String term,
  int? distance,
  bool? transpositionCostOne,
  bool? prefix,
  required BigInt topK,
  BigInt? offset,
  bool? enableSnippet,
});

// 4. Regex Search
Future<SearchResponse> searchDocumentsRegex({
  required String pattern,
  required BigInt topK,
  BigInt? offset,
  bool? enableSnippet,
});

// 5. Regex Phrase Search (ordered regex terms with proximity slop)
Future<SearchResponse> searchDocumentsRegexPhrase({
  required List<String> patterns,
  int? slop,
  int? maxExpansions,
  required BigInt topK,
  BigInt? offset,
  bool? enableSnippet,
});

// 6. More-Like-This Similar Document Search
Future<SearchResponse> searchSimilarDocuments({
  required String id,
  required BigInt topK,
  BigInt? minDocFrequency,
  BigInt? maxDocFrequency,
  BigInt? minTermFrequency,
  BigInt? minWordLength,
  BigInt? maxWordLength,
  double? boostFactor,
});

// 7. Field Existence Search ('id', 'title', or 'text')
Future<SearchResponse> searchDocumentsExists({
  required String fieldName,
  required BigInt topK,
  BigInt? offset,
});

// 8. Term Dictionary Prefix Autocomplete
Future<List<TermSuggestion>> suggestTerms({
  required String prefix,
  BigInt? limit,
  bool? includeTitle,
});
```

## Data Models

```dart
class Document {
  final String id;
  final String? title;
  final String text;
  const Document({required this.id, this.title, required this.text});
}

class TextRange {
  final BigInt start;
  final BigInt end;
  const TextRange({required this.start, required this.end});
}

class SearchResult {
  final double score;
  final Document doc;
  final String? snippet;
  final String? titleSnippet;
  final TextRange? fragmentRange;
  final List<TextRange>? highlightedRanges;
}

class SearchResponse {
  final BigInt totalHits;
  final List<SearchResult> results;
}

class LenientSearchResponse {
  final BigInt totalHits;
  final List<SearchResult> results;
  final List<String> errors;
}

class TermSuggestion {
  final String term;
  final int docFreq;
}

class IndexSpaceUsage {
  final BigInt totalBytes;
  final BigInt numDocs;
  final BigInt numSegments;
  final BigInt storeBytes;
  final BigInt postingsBytes;
  final BigInt positionsBytes;
  final BigInt fastFieldsBytes;
  final BigInt termdictBytes;
  final BigInt fieldnormsBytes;
  final BigInt deletesBytes;
}
```
