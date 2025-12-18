/**
 * @file A nix-inspired language
 * @author Whoman
 * @license GPLv3
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  expr: 1,
  function: 2,
  with: 3,
  unary: 4,
  binary: 5,
};

export default grammar({
  name: "tl",

  externals: ($) => [$.path],

  extras: ($) => [/\s/, $.comment],

  rules: {
    source_file: ($) => repeat($.expr),

    comment: (_) => token(seq("//", /.*/)),

    expr: ($) => choice($.binary_expr, $.unary_expr, $.postfix_expr),

    postfix_expr: ($) =>
      prec.right(
        PREC.expr,
        seq(
          $.primary,
          repeat(choice(field("call", $.call), $.member_access, $.array_index)),
        ),
      ),

    call: ($) =>
      seq(
        "(",
        optional(seq($.expr, repeat(seq(",", $.expr)), optional(","))),
        ")",
      ),

    // TODO: Add support for interpolation here
    member_access: ($) => seq(".", $.identifier),

    array_index: ($) => seq("[", $.expr, "]"),

    primary: ($) => choice($.literal, $.function, $.identifier, $.with_expr),

    with_expr: ($) => prec.left(PREC.with, seq($.with, $.expr, $.expr)),

    binary_expr: ($) =>
      prec.left(PREC.binary, seq($.expr, $.binary_operator, $.expr)),
    binary_operator: (_) =>
      token(
        choice(
          "+",
          "-",
          "*",
          "/",
          "%",
          "==",
          "!=",
          ">",
          ">=",
          "<",
          "<=",
          "&&",
          "||",
        ),
      ),
    unary_expr: ($) => prec.left(PREC.unary, seq($.unary_operator, $.expr)),
    unary_operator: (_) => token(choice("!", "+", "-")),

    literal: ($) =>
      choice($.null, $.number, $.string, $.boolean, $.path, $.array, $.object),

    null: (_) => token("null"),

    number: (_) => token(/\d+(\.\d+)?/),
    boolean: (_) => token(choice("true", "false")),

    string: ($) =>
      seq('"', repeat(choice(/./, $.interpolation, $.escape_sequence)), '"'),

    escape_sequence: (_) => seq("\\", choice('"', "\\", "n", "t")),

    interpolation: ($) => seq("${", field("expr", $.expr), "}"),

    array: ($) => seq("[", repeat($.expr), "]"),

    object: ($) => seq("{", repeat($.element), "}"),
    element: ($) => seq(field("key", $.expr), "=", field("value", $.expr)),

    function: ($) =>
      prec(
        PREC.function,
        seq(
          "|",
          repeat(seq($.identifier, optional(seq(":", $.type)), optional(","))),
          "|",
          optional(seq(":", $.type)),
          $.expr,
        ),
      ),

    identifier: ($) => choice(token(/[a-zA-Z_]\w*/), $.if),

    type: ($) =>
      choice(
        token("any"),
        token("nothing"),
        token("boolean"),
        token("int"),
        token("uint"),
        token("float"),
        token("number"),
        token("string"),
        token("path"),
        token("function"),
        seq(token("list"), "<", $.type, ">"),
        seq(token("object"), "<", repeat1(seq($.identifier, ":", $.type)), ">"),
        seq(token("either"), "<", repeat1(seq($.identifier, ",")), ">"),
        seq(token("thunk"), "<", $.type, ">"),
        $.expr,
      ),

    // Keywords
    with: (_) => token("with"),
    if: (_) => token("if"),
  },
});
