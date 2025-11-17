# DictNavi - English Dictionary Application

A Rust-based English dictionary application for word lookup and content display.

## Features

- Lookup English words from JSON-based dictionary files
- Display word definitions with phonetics and examples
- Simple command-line interface

## Project Structure

```
DictNavi/
├── Cargo.toml          # Project dependencies
├── src/
│   ├── main.rs         # Main application entry point
│   ├── lib.rs          # Library module exports
│   ├── models.rs       # Data structures for word definitions
│   └── dictionary.rs   # Dictionary lookup functionality
├── words/              # Directory containing word definition JSON files
│   ├── a.json          # Definition for word "a"
│   └── abandon.json    # Definition for word "abandon"
└── README.md           # This file
```

## Data Format

Word definitions are stored in JSON files with the following structure:

```json
{
  "word": "example",
  "phonetic": "/ɪɡˈzæmpəl/",
  "meanings": [
    {
      "partOfSpeech": "noun",
      "definitions": [
        {
          "definition": "A representative form or pattern",
          "example": "bad example"
        }
      ]
    }
  ]
}
```

## Getting Started

1. Make sure you have Rust installed (https://www.rust-lang.org/)
2. Clone this repository
3. Run the application:

```bash
cargo run
```

4. Enter words to look up their definitions
5. Type 'quit' to exit the application

## Adding New Words

To add new words to the dictionary:

1. Create a new JSON file in the `words/` directory
2. Name the file after the word (e.g., `example.json`)
3. Follow the JSON structure shown above
4. The word will be immediately available for lookup

## Dependencies

- `serde` - For JSON serialization/deserialization
- `serde_json` - For JSON parsing

## License

This project is licensed under the MIT License.
