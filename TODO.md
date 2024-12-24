- Tokenizer:
    - [] Add more binary operators.
    - [x] Do string interpolation in the tokenizer rather than in the AST.

- AST:
    - [x] Handle objects, arrays and other more complex expressions.
    - [] Handle struct and function declarations.
    - [x] Add logic operators.
    - [x] Use round parenthesis for order of operation in binary operations.
    - [x] Store location data for AST expressions/statements **(very important for error context)**.

- Type Checker:
    - [] Implement static type checking before evaluation.

- Runtime:
    - [] Add a way to inherit values and bring them into a scope, this would be especially useful for the std library so that It doesnt have to be redefined for each scope. Also make sure theres some way to mark variables as private so they cant be inherited.
    - [] Add an import function to the stdlib which evaluates another file and returns the evaluation result, this should be pretty simple but I know it won't be. 