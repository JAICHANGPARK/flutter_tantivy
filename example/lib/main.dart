import 'package:flutter/material.dart';
import 'package:flutter_tantivy/flutter_tantivy.dart';
import 'package:path_provider/path_provider.dart';
import 'dart:io';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Tantivy Demo',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.deepPurple,
          brightness: Brightness.light,
        ),
        useMaterial3: true,
      ),
      home: const TantivyDemoPage(),
    );
  }
}

class TantivyDemoPage extends StatefulWidget {
  const TantivyDemoPage({super.key});

  @override
  State<TantivyDemoPage> createState() => _TantivyDemoPageState();
}

class _TantivyDemoPageState extends State<TantivyDemoPage> {
  final TextEditingController _idController = TextEditingController();
  final TextEditingController _titleController = TextEditingController();
  final TextEditingController _textController = TextEditingController();
  final TextEditingController _searchController = TextEditingController();

  List<SearchResult> _searchResults = [];
  String _statusMessage = 'Ready';
  bool _isInitialized = false;
  bool _useRegexSearch = false;
  String _tokenizerType = 'cjk';
  BigInt _totalDocCount = BigInt.zero;

  @override
  void initState() {
    super.initState();
    _initializeTantivy();
  }

  Future<void> _initializeTantivy() async {
    try {
      final directory = await getApplicationDocumentsDirectory();
      final indexPath = '${directory.path}/tantivy_index';

      await Directory(indexPath).create(recursive: true);

      // Initialize Tantivy with CJK / N-gram tokenizer support by default
      initTantivy(dirPath: indexPath, tokenizerType: _tokenizerType);
      _updateDocCount();

      setState(() {
        _statusMessage =
            'Tantivy initialized ($dirPath: $indexPath, Tokenizer: $_tokenizerType)';
        _isInitialized = true;
      });
    } catch (e) {
      setState(() {
        _statusMessage = 'Initialization error: $e';
      });
    }
  }

  void _updateDocCount() {
    try {
      final count = getNumDocs();
      setState(() {
        _totalDocCount = count;
      });
    } catch (_) {}
  }

  Future<void> _addDocument() async {
    if (_idController.text.isEmpty || _textController.text.isEmpty) {
      setState(() {
        _statusMessage = 'Please enter both ID and Text';
      });
      return;
    }

    try {
      final doc = Document(
        id: _idController.text,
        title: _titleController.text.isNotEmpty ? _titleController.text : null,
        text: _textController.text,
      );

      await addDocument(doc: doc);
      _updateDocCount();

      setState(() {
        _statusMessage = 'Document added: [${doc.id}]';
      });

      _idController.clear();
      _titleController.clear();
      _textController.clear();
    } catch (e) {
      setState(() {
        _statusMessage = 'Add error: $e';
      });
    }
  }

  Future<void> _addSampleDocuments() async {
    try {
      final sampleDocs = [
        const Document(
          id: '1',
          title: 'Flutter UI Framework',
          text:
              'Flutter is an open-source UI software development kit created by Google.',
        ),
        const Document(
          id: '2',
          title: 'Rust Systems Language',
          text:
              'Rust is a multi-paradigm programming language focused on safety and performance.',
        ),
        const Document(
          id: '3',
          title: 'Tantivy Engine',
          text: 'Tantivy is a full-text search engine library written in Rust.',
        ),
        const Document(
          id: '4',
          title: '플러터 전문 검색 엔진',
          text: 'Tantivy 기반의 고성능 Flutter 전문 검색 라이브러리입니다.',
        ),
        const Document(
          id: '5',
          title: '한국어 N-gram 토큰화',
          text: '한글, 중국어, 일본어 텍스트 및 부분 문자열 검색을 완벽하게 지원합니다.',
        ),
      ];

      await addDocumentsBatch(docs: sampleDocs);
      _updateDocCount();

      setState(() {
        _statusMessage = 'Added ${sampleDocs.length} sample documents (English & CJK)';
      });
    } catch (e) {
      setState(() {
        _statusMessage = 'Batch add error: $e';
      });
    }
  }

  Future<void> _searchDocuments() async {
    if (_searchController.text.isEmpty) {
      setState(() {
        _statusMessage = 'Please enter search query or regex pattern';
      });
      return;
    }

    try {
      final SearchResponse response;
      if (_useRegexSearch) {
        response = await searchDocumentsRegex(
          pattern: _searchController.text,
          topK: BigInt.from(10),
          enableSnippet: true,
        );
      } else {
        response = await searchDocuments(
          query: _searchController.text,
          topK: BigInt.from(10),
          enableSnippet: true,
        );
      }

      setState(() {
        _searchResults = response.results;
        _statusMessage =
            'Found ${response.results.length} (Total matching: ${response.totalHits})';
      });
    } catch (e) {
      setState(() {
        _statusMessage = 'Search error: $e';
        _searchResults = [];
      });
    }
  }

  Future<void> _getDocumentById() async {
    if (_idController.text.isEmpty) {
      setState(() {
        _statusMessage = 'Please enter document ID';
      });
      return;
    }

    try {
      final doc = getDocumentById(id: _idController.text);

      if (doc != null) {
        setState(() {
          _statusMessage =
              'Found [${doc.id}]: ${doc.title != null ? "${doc.title} - " : ""}${doc.text}';
        });
      } else {
        setState(() {
          _statusMessage = 'Document not found: ${_idController.text}';
        });
      }
    } catch (e) {
      setState(() {
        _statusMessage = 'Get error: $e';
      });
    }
  }

  Future<void> _deleteDocument() async {
    if (_idController.text.isEmpty) {
      setState(() {
        _statusMessage = 'Please enter document ID';
      });
      return;
    }

    try {
      await deleteDocument(id: _idController.text);
      _updateDocCount();

      setState(() {
        _statusMessage = 'Document deleted: ${_idController.text}';
      });

      _idController.clear();
    } catch (e) {
      setState(() {
        _statusMessage = 'Delete error: $e';
      });
    }
  }

  Future<void> _clearAllDocuments() async {
    try {
      await deleteAllDocuments();
      _updateDocCount();
      setState(() {
        _searchResults = [];
        _statusMessage = 'All documents deleted from index';
      });
    } catch (e) {
      setState(() {
        _statusMessage = 'Clear error: $e';
      });
    }
  }

  @override
  void dispose() {
    _idController.dispose();
    _titleController.dispose();
    _textController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Flutter Tantivy Demo'),
        actions: [
          Center(
            child: Padding(
              padding: const EdgeInsets.only(right: 16),
              child: Chip(
                avatar: const Icon(Icons.storage, size: 16),
                label: Text('Docs: $_totalDocCount'),
              ),
            ),
          ),
        ],
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: !_isInitialized
          ? const Center(child: CircularProgressIndicator())
          : SingleChildScrollView(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  // Status Message
                  Card(
                    color: Colors.deepPurple.shade50,
                    child: Padding(
                      padding: const EdgeInsets.all(12),
                      child: Text(
                        _statusMessage,
                        style: TextStyle(
                          color: Colors.deepPurple.shade900,
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(height: 16),

                  // Actions & Samples
                  Row(
                    children: [
                      Expanded(
                        child: ElevatedButton.icon(
                          onPressed: _addSampleDocuments,
                          icon: const Icon(Icons.auto_awesome),
                          label: const Text('Add Samples (EN + CJK)'),
                          style: ElevatedButton.styleFrom(
                            padding: const EdgeInsets.all(14),
                          ),
                        ),
                      ),
                      const SizedBox(width: 8),
                      OutlinedButton.icon(
                        onPressed: _clearAllDocuments,
                        icon: const Icon(Icons.delete_sweep),
                        label: const Text('Clear All'),
                        style: OutlinedButton.styleFrom(
                          foregroundColor: Colors.red,
                          padding: const EdgeInsets.all(14),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 20),

                  // Input Form
                  const Text(
                    'Manage Document',
                    style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(height: 8),

                  Row(
                    children: [
                      Expanded(
                        flex: 1,
                        child: TextField(
                          controller: _idController,
                          decoration: const InputDecoration(
                            labelText: 'ID *',
                            border: OutlineInputBorder(),
                            isDense: true,
                          ),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        flex: 2,
                        child: TextField(
                          controller: _titleController,
                          decoration: const InputDecoration(
                            labelText: 'Title (Optional)',
                            border: OutlineInputBorder(),
                            isDense: true,
                          ),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),

                  TextField(
                    controller: _textController,
                    maxLines: 2,
                    decoration: const InputDecoration(
                      labelText: 'Text Content *',
                      border: OutlineInputBorder(),
                      isDense: true,
                    ),
                  ),
                  const SizedBox(height: 8),

                  Row(
                    children: [
                      Expanded(
                        child: ElevatedButton(
                          onPressed: _addDocument,
                          child: const Text('Add / Update'),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        child: OutlinedButton(
                          onPressed: _getDocumentById,
                          child: const Text('Get by ID'),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        child: OutlinedButton(
                          onPressed: _deleteDocument,
                          style: OutlinedButton.styleFrom(
                            foregroundColor: Colors.red,
                          ),
                          child: const Text('Delete ID'),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 24),

                  // Search Section
                  const Divider(),
                  const SizedBox(height: 12),

                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      const Text(
                        'Search Index',
                        style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
                      ),
                      FilterChip(
                        selected: _useRegexSearch,
                        label: Text(_useRegexSearch ? 'Regex Mode' : 'Text Mode'),
                        onSelected: (val) {
                          setState(() {
                            _useRegexSearch = val;
                          });
                        },
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),

                  TextField(
                    controller: _searchController,
                    decoration: InputDecoration(
                      labelText: _useRegexSearch ? 'Regex Pattern' : 'Query',
                      hintText: _useRegexSearch
                          ? 'e.g., 플러.* or Rust.*'
                          : 'e.g., 검색 OR Flutter',
                      border: const OutlineInputBorder(),
                      prefixIcon: const Icon(Icons.search),
                    ),
                    onSubmitted: (_) => _searchDocuments(),
                  ),
                  const SizedBox(height: 12),

                  ElevatedButton.icon(
                    onPressed: _searchDocuments,
                    icon: const Icon(Icons.search),
                    label: Text(
                      _useRegexSearch ? 'Run Regex Search' : 'Search Documents',
                    ),
                    style: ElevatedButton.styleFrom(
                      padding: const EdgeInsets.all(16),
                    ),
                  ),
                  const SizedBox(height: 20),

                  // Results List
                  if (_searchResults.isNotEmpty) ...[
                    Text(
                      'Results (${_searchResults.length}):',
                      style: const TextStyle(
                          fontSize: 16, fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 8),
                    ListView.builder(
                      shrinkWrap: true,
                      physics: const NeverScrollableScrollPhysics(),
                      itemCount: _searchResults.length,
                      itemBuilder: (context, index) {
                        final result = _searchResults[index];
                        return Card(
                          margin: const EdgeInsets.only(bottom: 8),
                          child: ListTile(
                            leading: CircleAvatar(
                              child: Text('${index + 1}'),
                            ),
                            title: Text(
                              result.doc.title != null &&
                                      result.doc.title!.isNotEmpty
                                  ? '[${result.doc.id}] ${result.doc.title}'
                                  : 'ID: ${result.doc.id}',
                              style:
                                  const TextStyle(fontWeight: FontWeight.bold),
                            ),
                            subtitle: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                const SizedBox(height: 4),
                                Text(result.doc.text),
                                if (result.snippet != null &&
                                    result.snippet!.isNotEmpty) ...[
                                  const SizedBox(height: 4),
                                  Text(
                                    'Snippet: ${result.snippet}',
                                    style: TextStyle(
                                      color: Colors.purple.shade700,
                                      fontStyle: FontStyle.italic,
                                    ),
                                  ),
                                ],
                                const SizedBox(height: 4),
                                Text(
                                  'Score: ${result.score.toStringAsFixed(4)}',
                                  style: TextStyle(
                                    color: Colors.blue.shade700,
                                    fontWeight: FontWeight.w500,
                                  ),
                                ),
                              ],
                            ),
                            isThreeLine: true,
                          ),
                        );
                      },
                    ),
                  ],
                ],
              ),
            ),
    );
  }
}
