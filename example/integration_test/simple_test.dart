import 'dart:io';
import 'package:integration_test/integration_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_tantivy/flutter_tantivy.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());

  test('Can call rust function and full Tantivy 0.26.2 API lifecycle', () async {
    expect(greet(name: "Tom"), "Hello, Tom!");

    final tempDir = await Directory.systemTemp.createTemp('tantivy_test_');
    try {
      initTantivy(dirPath: tempDir.path, tokenizerType: 'cjk');

      await addDocumentsBatch(
        docs: const [
          Document(
            id: '1',
            title: 'Flutter Framework',
            text: 'Flutter is an open-source UI toolkit created by Google.',
          ),
          Document(
            id: '2',
            title: '플러터 전문 검색 엔진',
            text: 'Tantivy 기반의 고성능 Flutter 전문 검색 라이브러리입니다.',
          ),
        ],
      );

      expect(getNumDocs(), BigInt.from(2));

      final searchRes = await searchDocuments(
        query: '검색',
        topK: BigInt.from(10),
        enableSnippet: true,
      );
      expect(searchRes.totalHits, BigInt.from(1));
      expect(searchRes.results.first.fragmentRange, isNotNull);

      final fuzzyRes = await searchDocumentsFuzzy(
        term: 'Fluter',
        distance: 1,
        topK: BigInt.from(10),
      );
      expect(fuzzyRes.totalHits, greaterThanOrEqualTo(BigInt.from(1)));

      final existsRes = await searchDocumentsExists(
        fieldName: 'title',
        topK: BigInt.from(10),
      );
      expect(existsRes.totalHits, BigInt.from(2));

      final usage = getIndexSpaceUsage();
      expect(usage.numDocs, BigInt.from(2));
      expect(usage.totalBytes, greaterThan(BigInt.zero));

      final segFiles = listIndexSegmentFiles();
      expect(segFiles, isNotEmpty);

      await deleteAllDocuments();
      expect(getNumDocs(), BigInt.zero);
      closeTantivy();
    } finally {
      if (await tempDir.exists()) {
        await tempDir.delete(recursive: true);
      }
    }
  });
}
