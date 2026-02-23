(comment) @comment

; Keywords
[
  (with)
  (if)
] @keyword

; Function parameters
(function (identifier) @variable.parameter)

; Function calls
(postfix_expr
  (primary
    (identifier) @function
    (#not-eq? @function "if"))
  .
  (call))
(postfix_expr
  (member_access
    (identifier) @function)
  .
  call: (call))

; Variables (simple identifier without call)
(postfix_expr
  (primary (identifier) @variable)
  !call)

; Object keys with simple identifiers (must come after variables)
(element
  key: (expr
    (postfix_expr
      (primary (identifier) @function))))

; Object keys with member access - highlight all identifiers
(element
  key: (expr
    (postfix_expr
      (primary (identifier) @function))))

(element
  key: (expr
    (postfix_expr
      (member_access (identifier) @function))))

; Type annotations in functions (must come after variables for precedence)
(function
  type: (expr
    (postfix_expr
      (primary (identifier) @type))))
(function
  type: (expr
    (postfix_expr
      (member_access (identifier) @type))))

; Literals
(null) @constant
(boolean) @constant.builtin.boolean
(number) @constant.builtin.numeric
(string) @string
(path) @string.special.path
(escape_sequence) @constant.character.escape

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
] @punctuation.bracket

; String interpolation
(interpolation
  "${" @punctuation.special
  (_) @embedded
  "}" @punctuation.special)
