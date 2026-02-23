/**
 * @file A nix-inspired language
 * @author Whoman
 * @license GPLv3
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  pipe: 0,
  with: 1,
  binary: 2,
  unary: 3,
  postfix: 4,
  function: 5,
};

export default grammar({
  name: "tl",

  externals: ($) => [$.path],

  conflicts: ($) => [[$.expr, $.pipe_expr]],

  extras: ($) => [/\s/, $.comment],

  rules: {
    source_file: ($) => repeat($.expr),

    comment: (_) => token(seq("//", /.*/)),

    expr: ($) =>
      choice(
        $.pipe_expr,
        $.binary_expr,
        $.unary_expr,
        $.postfix_expr,
        $.with_expr,
      ),

    pipe_expr: ($) =>
      prec.left(
        PREC.pipe,
        seq($.postfix_expr, repeat1(seq("|", $.postfix_expr))),
      ),

    postfix_expr: ($) =>
      prec.right(
        PREC.postfix,
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

    primary: ($) => choice($.literal, $.function, $.identifier),

    with_expr: ($) =>
      prec.left(
        PREC.with,
        seq($.with, field("variables", $.expr), field("body", $.expr)),
      ),

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
          repeat(
            seq(
              $.identifier,
              optional(seq(":", field("type", $.expr))),
              optional(","),
            ),
          ),
          "|",
          optional(seq(":", field("type", $.expr))),
          $.expr,
        ),
      ),

    identifier: ($) => choice(token(/[a-zA-Z_]\w*/), $.if),

    // Keywords
    with: (_) => token("with"),
    if: (_) => token("if"),
  },
});
