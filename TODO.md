- Tokenizer:
    - [x] Fix performance issues, parsing just the literal `1` takes ~20-30ms.
        - Fixed: ~2k lines parsed in under ~620µs.
    - [] Add more binary operators.

- AST:
    - [] Handle objects, arrays and other more complex expressions.
    - [] Handle struct and function declarations.

- [] Fix tests to use new APIs.