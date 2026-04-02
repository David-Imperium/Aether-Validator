# Supported Languages

Synward supports 24 programming languages:

## System Languages

| Language | Extensions | Parser |
|----------|------------|--------|
| Rust | .rs | tree-sitter-rust |
| C | .c, .h | tree-sitter-c |
| C++ | .cpp, .cc, .cxx, .hpp, .hxx | tree-sitter-cpp |
| CUDA | .cu, .cuh | tree-sitter-cuda |
| Go | .go | tree-sitter-go |
| Java | .java | tree-sitter-java |

## Scripting Languages

| Language | Extensions | Parser |
|----------|------------|--------|
| Python | .py, .pyi | tree-sitter-python |
| JavaScript | .js, .jsx, .mjs, .cjs | tree-sitter-javascript |
| TypeScript | .ts, .tsx, .mts, .cts | tree-sitter-typescript |
| Lua | .lua | tree-sitter-lua |
| Bash | .sh, .bash, .zsh, .ksh | tree-sitter-bash |

## Domain-Specific Languages

| Language | Extensions | Parser |
|----------|------------|--------|
| Lex | .lex | custom parser |
| Prism | .prism | tree-sitter-prism |
| GLSL | .glsl, .vert, .frag, .comp, .tesc, .tese, .geom | tree-sitter-glsl |
| Notebook | .ipynb | tree-sitter-json |
| CSS | .css | tree-sitter-css |
| HTML | .html, .htm | tree-sitter-html |
| SQL | .sql, .ddl, .dml | tree-sitter-sql |
| GraphQL | .graphql, .gql | tree-sitter-graphql |
| Markdown | .md, .markdown, .mdown, .mkd | tree-sitter-markdown |

## Configuration Languages

| Language | Extensions | Parser |
|----------|------------|--------|
| JSON | .json | tree-sitter-json |
| YAML | .yaml, .yml | tree-sitter-yaml |
| TOML | .toml | tree-sitter-toml-ng |
| CMake | .cmake | tree-sitter-cmake |

## Feature Support

All languages support:
- Parsing
- Syntax validation
- AST analysis
- Metrics calculation
- Certification

Some languages have additional features:
- Rust: unsafe detection, macro expansion
- Python: type hint analysis
- JavaScript/TypeScript: JSX/TSX support
