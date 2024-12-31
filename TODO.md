- Tokenizer:
    - [x] Add proper support for negative numbers.

- AST:
    - [-] Handle struct and function declarations.
        - Function declarations implemented.
    - [x] Add array indexing.

- Type Checker:
    - [] Implement static type checking before evaluation.

- Runtime:
    - [] Add a way to inherit values and bring them into a scope, this would be especially useful for the std library so that It doesnt have to be redefined for each scope. Also make sure theres some way to mark variables as private so they cant be inherited.
    - [] Add an import function to the stdlib which evaluates another file and returns the evaluation result, this should be pretty simple but I know it won't be. 