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
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
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
  final TextEditingController _textController = TextEditingController();
  final TextEditingController _searchController = TextEditingController();

  List<SearchResult> _searchResults = [];
  String _statusMessage = 'Ready';
  bool _isInitialized = false;

  @override
  void initState() {
    super.initState();
    _initializeTantivy();
  }

  Future<void> _initializeTantivy() async {
    try {
      final directory = await getApplicationDocumentsDirectory();
      final indexPath = '${directory.path}/tantivy_index';

      // 디렉토리 생성
      await Directory(indexPath).create(recursive: true);

      // Tantivy 초기화
      initTantivy(dirPath: indexPath);

      setState(() {
        _statusMessage = 'Tantivy initialized at: $indexPath';
        _isInitialized = true;
      });
    } catch (e) {
      setState(() {
        _statusMessage = 'Initialization error: $e';
      });
    }
  }

  Future<void> _addDocument() async {
    if (_idController.text.isEmpty || _textController.text.isEmpty) {
      setState(() {
        _statusMessage = 'Please enter both ID and text';
      });
      return;
    }

    try {
      final doc = Document(
        id: _idController.text,
        text: _textController.text,
      );

      await addDocument(doc: doc);

      setState(() {
        _statusMessage = 'Document added: ${doc.id}';
      });

      _idController.clear();
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
        const Document(id: '1', text: 'Flutter is an open-source UI software development kit created by Google'),
        const Document(id: '2', text: 'Rust is a multi-paradigm programming language focused on safety and performance'),
        const Document(id: '3', text: 'Tantivy is a full-text search engine library written in Rust'),
        const Document(id: '4', text: 'Flutter uses Dart programming language for building mobile applications'),
        const Document(id: '5', text: 'Flutter Rust Bridge enables Flutter apps to call Rust code efficiently'),
      ];

      await addDocumentsBatch(docs: sampleDocs);

      setState(() {
        _statusMessage = 'Added ${sampleDocs.length} sample documents';
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
        _statusMessage = 'Please enter search query';
      });
      return;
    }

    try {
      final results = await searchDocuments(
        query: _searchController.text,
        topK: BigInt.from(10),
      );

      setState(() {
        _searchResults = results;
        _statusMessage = 'Found ${results.length} results';
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
          _statusMessage = 'Found: ${doc.text}';
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

  @override
  void dispose() {
    _idController.dispose();
    _textController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Flutter Tantivy Demo'),
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
                    color: Colors.blue.shade50,
                    child: Padding(
                      padding: const EdgeInsets.all(12),
                      child: Text(
                        _statusMessage,
                        style: const TextStyle(fontWeight: FontWeight.w500),
                      ),
                    ),
                  ),
                  const SizedBox(height: 20),

                  // Add Sample Documents
                  ElevatedButton.icon(
                    onPressed: _addSampleDocuments,
                    icon: const Icon(Icons.auto_awesome),
                    label: const Text('Add Sample Documents'),
                    style: ElevatedButton.styleFrom(
                      padding: const EdgeInsets.all(16),
                    ),
                  ),
                  const SizedBox(height: 20),

                  // Document ID Input
                  TextField(
                    controller: _idController,
                    decoration: const InputDecoration(
                      labelText: 'Document ID',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.key),
                    ),
                  ),
                  const SizedBox(height: 12),

                  // Document Text Input
                  TextField(
                    controller: _textController,
                    maxLines: 3,
                    decoration: const InputDecoration(
                      labelText: 'Document Text',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.text_fields),
                    ),
                  ),
                  const SizedBox(height: 12),

                  // CRUD Buttons
                  Row(
                    children: [
                      Expanded(
                        child: ElevatedButton(
                          onPressed: _addDocument,
                          child: const Text('Add'),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        child: ElevatedButton(
                          onPressed: _getDocumentById,
                          child: const Text('Get'),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        child: ElevatedButton(
                          onPressed: _deleteDocument,
                          style: ElevatedButton.styleFrom(
                            backgroundColor: Colors.red.shade400,
                            foregroundColor: Colors.white,
                          ),
                          child: const Text('Delete'),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 24),

                  // Search Section
                  const Divider(),
                  const SizedBox(height: 12),
                  const Text(
                    'Search',
                    style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(height: 12),

                  TextField(
                    controller: _searchController,
                    decoration: const InputDecoration(
                      labelText: 'Search Query',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.search),
                      hintText: 'e.g., Flutter OR Rust',
                    ),
                    onSubmitted: (_) => _searchDocuments(),
                  ),
                  const SizedBox(height: 12),

                  ElevatedButton.icon(
                    onPressed: _searchDocuments,
                    icon: const Icon(Icons.search),
                    label: const Text('Search Documents'),
                    style: ElevatedButton.styleFrom(
                      padding: const EdgeInsets.all(16),
                    ),
                  ),
                  const SizedBox(height: 20),

                  // Search Results
                  if (_searchResults.isNotEmpty) ...[
                    const Text(
                      'Search Results:',
                      style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
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
                              'ID: ${result.doc.id}',
                              style: const TextStyle(fontWeight: FontWeight.bold),
                            ),
                            subtitle: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                const SizedBox(height: 4),
                                Text(result.doc.text),
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
