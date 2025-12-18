(comment) @comment

; Keywords
[
  (with)
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

; Function bindings
(element
  key: (expr (postfix_expr (primary (identifier) @function)))
  value: (expr (postfix_expr (primary (function)))))

; Function parameters
(function (identifier) @variable.parameter)

; Literals
(null) @constant
(boolean) @constant.builtin.boolean
(number) @constant.builtin.numeric
(string) @string
(path) @string.special.path
(escape_sequence) @constant.character.escape

; Types
[
  "any"
  "nothing"
  "boolean"
  "int"
  "uint"
  "float"
  "number"
  "string"
  "path"
  "function"
  "list"
  "object"
  "either"
  "thunk"
  (type)
] @type

(type (expr (postfix_expr (primary
  (identifier) @type)
  !call)))

; Operators
(binary_operator) @operator
(unary_operator) @operator

; Punctuation
[
  "."
  ","
  ":"
  "|"
  "="
] @punctuation.delimiter

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
  "<"
  ">"
] @punctuation.bracket

; Has to be after the bracket punctuation decl for the closing bracket to be highlighted properly
(interpolation
  "${" @punctuation.special
  (expr) @embedded
  "}" @punctuation.special)

