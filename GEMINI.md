# Project Overview

This project is a Flutter plugin named `flutter_tantivy`. It provides full-text search capabilities to Flutter applications by leveraging the power of the Tantivy search engine library, which is written in Rust.

The project is structured as a multi-language project, combining Dart/Flutter for the application layer and Rust for the high-performance search functionality. The integration between Dart and Rust is managed by `flutter_rust_bridge`, which automatically generates the necessary glue code for foreign function interface (FFI) calls.

The core of the project resides in two main directories:

-   `lib/`: Contains the Dart code that defines the plugin's public API and interacts with the native Rust code.
-   `rust/`: Contains the Rust crate that wraps the Tantivy library and exposes the search functionality to the Dart side.

The project also includes an `example/` directory with a sample Flutter application demonstrating how to use the plugin.

## Building and Running

The project uses a custom build tool located in the `cargokit/` directory to manage the compilation of the Rust code and its integration with the Flutter build process.

### Building the Rust Crate

To build the Rust crate, you can use the standard Cargo commands within the `rust/` directory:

```bash
cd rust
cargo build
```

### Running the Example Application

To run the example application, you can use the standard Flutter commands from the `example/` directory:

```bash
cd example
flutter run
```

The build tool in `cargokit/` should automatically handle the compilation of the Rust code and its inclusion in the final application.

## Development Conventions

### Code Generation

The project relies on `flutter_rust_bridge` for code generation. When you modify the Rust code in `rust/src/api.rs`, you need to run the code generator to update the Dart bindings. The configuration for the code generator is in `flutter_rust_bridge.yaml`.

To run the code generator, you can use the following command:

```bash
flutter_rust_bridge_codegen
```

### Testing

The project includes an integration test in `test_driver/integration_test.dart`. You can run the test using the following command:

```bash
cd example
flutter drive --driver=test_driver/integration_test.dart --target=integration_test/app_test.dart
```

### Dependencies

The project's dependencies are managed by `pub` for the Dart part and `cargo` for the Rust part. The Dart dependencies are listed in `pubspec.yaml`, and the Rust dependencies are in `rust/Cargo.toml`.
