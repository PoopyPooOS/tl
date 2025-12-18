(comment) @comment

; Keywords
[
  (with)
  (let)
  (in)
  (if)
] @keyword

; Functions
(postfix_expr
  (primary
    (identifier) @function
    (#not-eq? @function "if"))
  (call))

; Identifiers
(postfix_expr
  (primary (identifier) @variable)
  !call)

(object key: (identifier) @variable)

(object
  key: (identifier) @function
  value: (expr
    (postfix_expr
      (primary
        (function)))))

(binding name: (identifier) @variable)

(binding
  name: (identifier) @function
  expr: (expr
    (postfix_expr
      (primary
        (function)))))

(function argument: (identifier) @variable.parameter)

; Literals
(null) @constant
(boolean) @constant.builtin.boolean
(number) @constant.builtin.numeric
(string) @string
(path) @string.special.path
(escape_sequence) @constant.character.escape
(interpolation
  "${" @punctuation.special
  expr: (_) @embedded
  "}" @punctuation.special)

; Operators
(binary_operator) @operator
(unary_operator) @operator

; Symbols
[
  (dot)
  (comma)
  (equals)
  (colon)
] @punctuation.delimiter
(bracket) @punctuation.bracket
