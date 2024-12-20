- Tokenizer:
    - [] Add more binary operators.

- AST:
    - [x] Handle objects, arrays and other more complex expressions.
    - [-] Handle struct and function declarations.
        - Function declaration implemented.
    - [] Use round parenthesis for order of operation in binary ops.
    - [] Store location data for AST expressions/statements **(very important for error context)**.

- Type Checker:
    - [] Implement static type checking before evaluation.

- Runtime:
    - [] Add a way to inherit values and bring them into a scope, this would be especially useful for the std library so that It doesnt have to be redefined for each scope. Also make sure theres some way to mark variables as private so they cant be inherited.