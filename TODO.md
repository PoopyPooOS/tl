- [x] Make location information more descriptive, instead of `line` and `start..=end` have `(line, start)..=(line, end)` (like tree-sitter) or similar.

- Tree-sitter grammar:
    - [x] Update function call rules to not require commas.
    - [x] Make it so that there can be multiple expressions without any errors.

- Runtime:
    - [] Implement field access.
    - [x] Use safer arthimetic when evaluating binary operations.
    - [] Add an import function to the stdlib which evaluates another file and returns the evaluation result. 
