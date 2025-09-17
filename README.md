# screen-ocr

`screen-ocr` is a Rust-based application for performing Optical Character Recognition (OCR) directly from your screen. It allows users to select an area of their screen and extract text from it, providing a convenient way to digitize text from images or non-selectable areas. Supports both Windows and Linux operating systems.

## Installation

To install `screen-ocr`, you'll need to have Rust and Cargo installed on your system. If you don't have them, you can install them by following the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).

### From Source

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/alliby/screen-ocr.git
    cd screen-ocr
    ```

2.  **Build the application:**
    ```bash
    cargo build --release
    ```

3.  **Run the application:**
    ```bash
    ./target/release/screen-ocr
    ```

### Pre-built Binaries

Pre-built binaries for Windows and Linux are available on the [GitHub Releases page](https://github.com/alliby/screen-ocr/releases).

## Contributing

Contributions are welcome! If you have any bug reports, feature requests, or want to contribute code, please feel free to open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).
