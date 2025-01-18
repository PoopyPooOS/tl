- [] Make location information more descriptive, instead of `line` and `start..=end` have `(line, start)..=(line, end)` (like tree-sitter) or similar.

- AST:
    - [] Add more explicit location information for function calls

- Tree-sitter grammar:
    - [] Update function call rules to not require commas.

- Runtime:
    - [] Implement field access.
    - [] Add an import function to the stdlib which evaluates another file and returns the evaluation result. 
