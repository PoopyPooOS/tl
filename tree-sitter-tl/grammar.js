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
  let_in: 4,
  unary: 5,
  binary: 6,
};

export default grammar({
  name: "tl",

  externals: ($) => [$.path],

  extras: ($) => [/\s/, $.comment],

  rules: {
    source_file: ($) => $.expr,

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
        alias("(", $.bracket),
        repeat(seq($.expr, optional(alias(",", $.comma)))),
        alias(")", $.bracket),
      ),

    // TODO: Add support for interpolation here
    member_access: ($) => seq(alias(".", $.dot), $.identifier),

    array_index: ($) =>
      seq(alias("[", $.bracket), $.expr, alias("]", $.bracket)),

    primary: ($) =>
      choice($.literal, $.function, $.identifier, $.with_expr, $.let_in),

    binding: ($) =>
      seq(
        field("name", $.identifier),
        alias("=", $.equals),
        field("expr", $.expr),
      ),

    let_in: ($) =>
      prec(PREC.let_in, seq($.let, repeat($.binding), $.in, $.expr)),

    with_expr: ($) => prec(PREC.with, seq($.with, $.expr, $.expr)),

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

    array: ($) =>
      seq(alias("[", $.bracket), repeat($.expr), alias("]", $.bracket)),

    object: ($) =>
      seq(
        alias("{", $.bracket),
        repeat(
          seq(
            field("key", $.identifier),
            alias("=", $.equals),
            field("value", $.expr),
          ),
        ),
        alias("}", $.bracket),
      ),

    function: ($) =>
      prec(
        PREC.function,
        seq(
          field("argument", $.identifier),
          alias(":", $.colon),
          field("expr", $.expr),
        ),
      ),

    identifier: ($) => choice(token(/[a-zA-Z_]\w*/), $.if),

    // Keywords
    with: (_) => token("with"),
    let: (_) => token("let"),
    in: (_) => token("in"),
    if: (_) => token("if"),
  },
});
