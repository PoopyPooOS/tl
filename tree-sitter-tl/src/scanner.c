#include <tree_sitter/parser.h>
#include <wctype.h>

enum TokenType {
  PATH_TOKEN
};

static bool scan_interpolation(TSLexer *lexer) {
  if (lexer->lookahead != '{') return false;
  lexer->advance(lexer, false); // consume '{'

  int depth = 1;
  while (depth > 0) {
    if (lexer->lookahead == 0 || lexer->lookahead == '\n') return false;

    if (lexer->lookahead == '{') {
      depth++;
    } else if (lexer->lookahead == '}') {
      depth--;
    }
    lexer->advance(lexer, false);
  }
  return true;
}

static bool is_path_char(int32_t c) {
  return c != 0 &&
         c != '\n' &&
         c != '\r' &&
         c != ' ' &&
         c != '\t' &&
         c != '(' &&
         c != ')' &&
         c != '"' &&
         c != '\'';
}

bool tree_sitter_tl_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  if (!valid_symbols[PATH_TOKEN]) return false;

  // Prevent capturing comments as paths
  if (lexer->lookahead == '/') {
    lexer->mark_end(lexer);
    lexer->advance(lexer, false);
  
    if (lexer->lookahead == '/' || lexer->lookahead == '*') {
      return false;
    }
  }

  while (iswspace(lexer->lookahead) && lexer->lookahead != '\n') {
    lexer->advance(lexer, true);
  }

  bool seen_slash = false;

  while (is_path_char(lexer->lookahead)) {
    if (lexer->lookahead == '/') {
      seen_slash = true;
      lexer->advance(lexer, false);
    } else if (lexer->lookahead == '$') {
      lexer->advance(lexer, false);
      if (!scan_interpolation(lexer)) return false;
    } else {
      lexer->advance(lexer, false);
    }
  }

  if (!seen_slash) return false;

  lexer->result_symbol = PATH_TOKEN;
  return true;
}

void *tree_sitter_tl_external_scanner_create() { return NULL; }
void tree_sitter_tl_external_scanner_destroy(void *p) {}
unsigned tree_sitter_tl_external_scanner_serialize(void *p, char *b) { return 0; }
void tree_sitter_tl_external_scanner_deserialize(void *p, const char *b, unsigned n) {}
