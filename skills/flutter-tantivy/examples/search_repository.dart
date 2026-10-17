import 'package:flutter_tantivy/flutter_tantivy.dart';
import 'package:path_provider/path_provider.dart';

/// Example repository encapsulating `flutter_tantivy` initialization,
/// indexing, lenient/fuzzy search, similar document lookup, and disposal.
class LocalSearchRepository {
  bool _initialized = false;

  Future<void> init({String tokenizerType = 'cjk'}) async {
    if (_initialized) return;
    await RustLib.init();
    final directory = await getApplicationDocumentsDirectory();
    initTantivy(
      dirPath: '${directory.path}/tantivy_index',
      tokenizerType: tokenizerType,
    );
    _initialized = true;
  }

  Future<void> indexDocuments(List<Document> documents) async {
    await addDocumentsBatch(docs: documents);
  }

  /// Safe end-user search that boosts title matches and falls back to fuzzy search
  /// when no exact/lenient matches are found.
  Future<List<SearchResult>> search(
    String userQuery, {
    int limit = 20,
    int offset = 0,
  }) async {
    final trimmed = userQuery.trim();
    if (trimmed.isEmpty) return const [];

    final lenient = await searchDocumentsLenient(
      query: trimmed,
      topK: BigInt.from(limit),
      offset: BigInt.from(offset),
      enableSnippet: true,
      titleBoost: 2.0,
    );

    if (lenient.results.isNotEmpty) {
      return lenient.results;
    }

    final fuzzy = await searchDocumentsFuzzy(
      term: trimmed,
      distance: 1,
      transpositionCostOne: true,
      prefix: true,
      topK: BigInt.from(limit),
      offset: BigInt.from(offset),
      enableSnippet: true,
    );
    return fuzzy.results;
  }

  Future<List<TermSuggestion>> autocomplete(String prefix, {int limit = 8}) {
    return suggestTerms(
      prefix: prefix,
      limit: BigInt.from(limit),
      includeTitle: true,
    );
  }

  Future<List<SearchResult>> findRelated(String docId, {int limit = 5}) async {
    final res = await searchSimilarDocuments(
      id: docId,
      topK: BigInt.from(limit),
    );
    return res.results;
  }

  IndexSpaceUsage getStorageStats() => getIndexSpaceUsage();

  void dispose() {
    if (!_initialized) return;
    closeTantivy();
    _initialized = false;
  }
}
