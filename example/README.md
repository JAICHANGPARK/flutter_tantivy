# flutter_tantivy Example

This example demonstrates how to use the `flutter_tantivy` plugin to integrate full-text search capabilities powered by Tantivy into your Flutter application.

## Features Demonstrated

This example app showcases all the major features of the flutter_tantivy plugin:

- ✅ **Index Initialization**: Setting up a persistent search index
- ✅ **Adding Documents**: Single and batch document insertion
- ✅ **Searching**: Full-text search with relevance scoring
- ✅ **Retrieving Documents**: Getting documents by ID
- ✅ **Deleting Documents**: Single document deletion
- ✅ **Sample Data**: Quick loading of sample documents for testing

## Getting Started

### Prerequisites

- Flutter SDK (3.3.0 or higher)
- Rust toolchain (for building native code)

### Running the Example

1. Navigate to the example directory:
```bash
cd example
```

2. Get dependencies:
```bash
flutter pub get
```

3. Run the app:
```bash
flutter run
```

## Using the Demo App

### 1. Initialization
When you launch the app, it automatically:
- Initializes the Tantivy search engine
- Creates an index directory in the app's documents folder
- Displays the index path in the status message

### 2. Adding Sample Documents
- Tap **"Add Sample Documents"** to load 5 sample documents about Flutter, Rust, and Tantivy
- The status message will confirm how many documents were added

Sample documents include:
- Flutter UI toolkit information
- Rust programming language description
- Tantivy search engine details
- Flutter and Dart relationship
- Flutter Rust Bridge information

### 3. Adding Custom Documents
1. Enter a unique **Document ID** (e.g., "6", "myDoc")
2. Enter the **Document Text** (the content you want to search)
3. Tap **"Add"** to insert the document

### 4. Searching Documents
1. Enter a search query in the **Search Query** field
   - Examples:
     - `Flutter` - Find documents containing "Flutter"
     - `Rust` - Find documents about Rust
     - `Flutter OR Rust` - Find documents with either term
     - `Flutter AND Dart` - Find documents with both terms
2. Tap **"Search Documents"** or press Enter
3. View results with relevance scores below

### 5. Retrieving by ID
1. Enter a **Document ID** in the ID field
2. Tap **"Get"** to retrieve that specific document
3. The document text will appear in the status message

### 6. Deleting Documents
1. Enter a **Document ID** in the ID field
2. Tap **"Delete"** to remove the document
3. The status message confirms deletion

## Code Overview

### Main Components

#### TantivyDemoPage
The main page widget that contains all the UI and search functionality.

#### Key Methods

**_initializeTantivy()**
```dart
// Initializes the Tantivy index at app startup
final directory = await getApplicationDocumentsDirectory();
final indexPath = '${directory.path}/tantivy_index';
initTantivy(dirPath: indexPath);
```

**_addDocument()**
```dart
// Adds a single document to the index
final doc = Document(id: _idController.text, text: _textController.text);
await addDocument(doc: doc);
```

**_addSampleDocuments()**
```dart
// Efficiently adds multiple documents in one batch
await addDocumentsBatch(docs: sampleDocs);
```

**_searchDocuments()**
```dart
// Searches the index and displays results with scores
final results = await searchDocuments(
  query: _searchController.text,
  topK: BigInt.from(10),
);
```

**_getDocumentById()**
```dart
// Retrieves a specific document by its ID (synchronous)
final doc = getDocumentById(id: _idController.text);
```

**_deleteDocument()**
```dart
// Deletes a document from the index
await deleteDocument(id: _idController.text);
```

### UI Structure

The app uses a scrollable single-page layout with:
- Status card showing operation results
- Sample data button
- Input fields for ID and text
- CRUD operation buttons (Add, Get, Delete)
- Search section with query input
- Results list showing matched documents with scores

## Performance Tips

When building your own app:

1. **Use Batch Operations**: The example shows how `addDocumentsBatch` is more efficient than adding documents one by one

2. **Persistent Index**: The index is stored in the documents directory, so data persists across app restarts

3. **Async Operations**: Most operations are async to prevent UI blocking

4. **Sync Operations**: `getDocumentById` is synchronous for quick lookups

## Customization

### Adding More Fields

To add more fields to your documents, you'll need to modify the Rust code:

1. Update `rust/src/api/tantivy_api.rs` to add new fields to the schema
2. Regenerate bindings with `flutter_rust_bridge_codegen generate`
3. Update the Dart code to use the new fields

### Custom UI

Feel free to customize the UI in `lib/main.dart`:
- Change colors and themes
- Add more search options
- Implement search filters
- Add pagination for results

## Testing Search Queries

Try these queries with the sample documents:

| Query | Expected Results |
|-------|------------------|
| `Flutter` | 3 documents (about Flutter, Dart, Flutter Rust Bridge) |
| `Rust` | 3 documents (about Rust, Tantivy, Flutter Rust Bridge) |
| `Google` | 1 document (about Flutter by Google) |
| `search engine` | 1 document (about Tantivy) |
| `Flutter OR Rust` | 5 documents (all samples) |
| `Flutter AND Google` | 1 document (Flutter by Google) |

## Troubleshooting

### Index initialization fails
- Ensure the app has permission to write to the documents directory
- Check device storage space

### Search returns no results
- Verify documents were added successfully
- Check the search query syntax
- Ensure the index was initialized

### Build errors
- Run `flutter clean` and `flutter pub get`
- Ensure Rust toolchain is properly installed
- Check that all platform-specific dependencies are met

## Learn More

- [flutter_tantivy Plugin Documentation](../README.md)
- [Tantivy Documentation](https://docs.rs/tantivy/)
- [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge)

## License

This example is part of the flutter_tantivy package and is licensed under the MIT License.
