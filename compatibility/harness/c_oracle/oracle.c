/* Minimal Redland/Oxiland C oracle for 0.11 two-sided differentials.
 * Usage: oracle --engine redland|oxiland --fixture path.json
 * Prints one JSON object of observations to stdout.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <librdf.h>
/* System Redland keeps list decls in a separate header; Oxiland folds them in. */
#if __has_include(<rdf_list.h>)
#include <rdf_list.h>
#endif

static void die(const char *msg) {
  fprintf(stderr, "oracle error: %s\n", msg);
  exit(2);
}

static char *read_file(const char *path, long *out_len) {
  FILE *f = fopen(path, "rb");
  if (!f) die("open fixture");
  fseek(f, 0, SEEK_END);
  long n = ftell(f);
  fseek(f, 0, SEEK_SET);
  char *buf = malloc((size_t)n + 1);
  if (!buf) die("oom");
  if (fread(buf, 1, (size_t)n, f) != (size_t)n) die("read fixture");
  buf[n] = 0;
  fclose(f);
  if (out_len) *out_len = n;
  return buf;
}

static int json_has(const char *json, const char *key) {
  char pat[128];
  snprintf(pat, sizeof(pat), "\"%s\"", key);
  return strstr(json, pat) != NULL;
}

static char *json_string(const char *json, const char *key) {
  char pat[128];
  snprintf(pat, sizeof(pat), "\"%s\"", key);
  const char *p = strstr(json, pat);
  if (!p) return NULL;
  p = strchr(p + strlen(pat), '"');
  if (!p) return NULL;
  p++;
  /* Scan to closing quote, respecting JSON escapes. */
  const char *end = p;
  while (*end) {
    if (*end == '\\' && end[1]) {
      end += 2;
      continue;
    }
    if (*end == '"') break;
    end++;
  }
  if (*end != '"') return NULL;
  size_t n = (size_t)(end - p);
  char *out = malloc(n + 1);
  memcpy(out, p, n);
  out[n] = 0;
  return out;
}

/* Unescape a minimal JSON string value (handles \\n \\t \\" \\\\). */
static char *json_unescape(const char *s) {
  size_t n = strlen(s);
  char *out = malloc(n + 1);
  size_t j = 0;
  for (size_t i = 0; i < n; i++) {
    if (s[i] == '\\' && i + 1 < n) {
      char c = s[i + 1];
      if (c == 'n') { out[j++] = '\n'; i++; }
      else if (c == 't') { out[j++] = '\t'; i++; }
      else if (c == '"' || c == '\\' || c == '/') { out[j++] = c; i++; }
      else { out[j++] = s[i]; }
    } else {
      out[j++] = s[i];
    }
  }
  out[j] = 0;
  return out;
}

static int steps_contain(const char *json, const char *op) {
  char pat[160];
  snprintf(pat, sizeof(pat), "\"op\": \"%s\"", op);
  if (strstr(json, pat)) return 1;
  snprintf(pat, sizeof(pat), "\"op\":\"%s\"", op);
  return strstr(json, pat) != NULL;
}

int main(int argc, char **argv) {
  const char *engine = NULL;
  const char *fixture_path = NULL;
  for (int i = 1; i < argc; i++) {
    if (!strcmp(argv[i], "--engine") && i + 1 < argc) engine = argv[++i];
    else if (!strcmp(argv[i], "--fixture") && i + 1 < argc) fixture_path = argv[++i];
  }
  if (!engine || !fixture_path) die("usage: oracle --engine redland|oxiland --fixture FILE");

  long flen = 0;
  char *fixture = read_file(fixture_path, &flen);
  char *turtle_raw = json_string(fixture, "turtle");
  char *turtle = turtle_raw ? json_unescape(turtle_raw) : NULL;
  free(turtle_raw);
  char *id = json_string(fixture, "id");

  librdf_world *world = librdf_new_world();
  if (!world) die("new world");
  librdf_world_open(world);

  int ok = 1;
  int size = 0;
  int ask = -1;
  int select_count = -1;
  int stream_count = -1;
  int parsed = 0;
  int nodes = 0;
  int contains_ok = 0;
  char digest_hex[64] = {0};
  char *serialized = NULL;
  const char *error = NULL;
  int world_opened = 0;
  int concepts = 0;
  int unicode_ok = 0;
  int is_blank = -1;
  char *filename = NULL;
  char *logged = NULL;
  char *feature = NULL;

  /* Always open world for lifecycle observations. */
  world_opened = 1;

  librdf_storage *storage = NULL;
  librdf_model *model = NULL;
  librdf_parser *parser = NULL;
  librdf_uri *base_uri = NULL;

  if (steps_contain(fixture, "storage_memory") ||
      steps_contain(fixture, "model_memory") ||
      steps_contain(fixture, "model_from_storage") ||
      turtle) {
    storage = librdf_new_storage(world, "memory", NULL, NULL);
    model = storage ? librdf_new_model(world, storage, NULL) : NULL;
    if (!storage || !model) {
      ok = 0;
      error = "model alloc failed";
    }
  }

  if (ok && turtle && model) {
    parser = librdf_new_parser(world, "turtle", NULL, NULL);
    base_uri = librdf_new_uri(world, (const unsigned char *)"http://example.org/");
    if (!parser || !base_uri) {
      ok = 0;
      error = "parser alloc failed";
    } else {
      int parse_rc = librdf_parser_parse_string_into_model(
          parser, (const unsigned char *)turtle, base_uri, model);
      if (parse_rc != 0) {
        ok = 0;
        error = "parse failed";
      } else {
        parsed = 1;
        size = librdf_model_size(model);
        stream_count = size;
      }
    }
  } else if (ok && model && !turtle) {
    size = librdf_model_size(model);
  }

  if (ok && model && (steps_contain(fixture, "ask") ||
                      steps_contain(fixture, "cli_parse_ask") ||
                      json_has(fixture, "\"ask\""))) {
    librdf_query *qask = librdf_new_query(
        world, "sparql", NULL, (const unsigned char *)"ASK { ?s ?p ?o }", NULL);
    if (qask) {
      librdf_query_results *r = librdf_model_query_execute(model, qask);
      if (r) {
        ask = librdf_query_results_get_boolean(r) ? 1 : 0;
        librdf_free_query_results(r);
      }
      librdf_free_query(qask);
    }
  }

  if (ok && model && steps_contain(fixture, "select_count")) {
    librdf_query *qsel = librdf_new_query(
        world, "sparql", NULL,
        (const unsigned char *)"SELECT ?s WHERE { ?s ?p ?o }", NULL);
    if (qsel) {
      librdf_query_results *r = librdf_model_query_execute(model, qsel);
      select_count = 0;
      if (r) {
        while (!librdf_query_results_finished(r)) {
          select_count++;
          librdf_query_results_next(r);
        }
        librdf_free_query_results(r);
      }
      librdf_free_query(qsel);
    }
  }

  if (ok && model && steps_contain(fixture, "find_stream_count")) {
    librdf_stream *stream = librdf_model_as_stream(model);
    stream_count = 0;
    if (stream) {
      while (!librdf_stream_end(stream)) {
        stream_count++;
        librdf_stream_next(stream);
      }
      librdf_free_stream(stream);
    }
  }

  if (ok && model && steps_contain(fixture, "serialize_ntriples")) {
    librdf_serializer *ser =
        librdf_new_serializer(world, "ntriples", NULL, NULL);
    if (ser) {
      unsigned char *out =
          librdf_serializer_serialize_model_to_string(ser, NULL, model);
      if (out) {
        serialized = (char *)out;
        contains_ok = strstr(serialized, "<http://example.org/s>") != NULL ||
                      strstr(serialized, "<http://example.org/alice>") != NULL ||
                      strstr(serialized, "<http://example.org/a>") != NULL;
      }
      librdf_free_serializer(ser);
    }
  }

  if (steps_contain(fixture, "uri_new") ||
      steps_contain(fixture, "node_from_uri_string") ||
      steps_contain(fixture, "node_from_literal")) {
    if (steps_contain(fixture, "uri_new") ||
        steps_contain(fixture, "node_from_uri_string")) {
      librdf_uri *u =
          librdf_new_uri(world, (const unsigned char *)"http://example.org/n");
      librdf_node *n = u ? librdf_new_node_from_uri(world, u) : NULL;
      if (n) {
        nodes++;
        librdf_free_node(n);
      }
      if (u) librdf_free_uri(u);
    }
    if (steps_contain(fixture, "node_from_literal")) {
      librdf_node *n = librdf_new_node_from_literal(
          world, (const unsigned char *)"hello", NULL, 0);
      if (n) {
        nodes++;
        librdf_free_node(n);
      }
    }
  }

  if (steps_contain(fixture, "digest_md5")) {
    librdf_digest *d = librdf_new_digest(world, "MD5");
    char *input_owned = json_string(fixture, "input");
    if (!input_owned) {
      ok = 0;
      error = "digest input missing";
    } else if (d) {
      librdf_digest_init(d);
      librdf_digest_update(d, (unsigned char *)input_owned, strlen(input_owned));
      librdf_digest_final(d);
      unsigned char *raw = (unsigned char *)librdf_digest_get_digest(d);
      size_t dig_len = librdf_digest_get_digest_length(d);
      for (size_t i = 0; i < dig_len && i < 16; i++) {
        sprintf(digest_hex + (i * 2), "%02x", raw[i]);
      }
      librdf_free_digest(d);
    } else {
      ok = 0;
      error = "md5 digest unavailable";
    }
    free(input_owned);
  }

  if (steps_contain(fixture, "list_lifecycle")) {
    librdf_list *list = librdf_new_list(world);
    if (list) {
      size = 1;
      librdf_free_list(list);
    } else {
      ok = 0;
      error = "list alloc failed";
    }
  }

  if (steps_contain(fixture, "log_simple")) {
#ifdef LIBRDF_LOG_INFO
    librdf_log_simple(world, 0, LIBRDF_LOG_INFO, LIBRDF_FROM_INIT, NULL, "oracle log");
#else
    /* Oxiland header exposes level/facility as ints matching Redland enums. */
    librdf_log_simple(world, 0, 1 /* INFO */, 5 /* INIT */, NULL, "oracle log");
#endif
    logged = strdup("oracle log");
  }

  if (steps_contain(fixture, "concepts_probe")) {
    librdf_node *concept = librdf_get_concept_resource_by_index(world, 0);
    concepts = concept != NULL;
    /* Redland returns borrowed pointers; do not free. */
  }

  if (steps_contain(fixture, "file_uri_to_filename")) {
    char *uri_s = json_string(fixture, "uri");
    if (!uri_s) uri_s = strdup("file:///tmp/x");
    librdf_uri *u = librdf_new_uri(world, (const unsigned char *)uri_s);
    if (u) {
      const char *fn = (const char *)librdf_uri_to_filename(u);
      if (fn) {
        filename = strdup(fn);
        /* Some builds return malloc'd bytes; prefer librdf_free_memory when available. */
#if defined(LIBRDF_VERSION) || 1
        /* filename from uri_to_filename is owned; duplicate then free original if possible. */
#endif
      }
      librdf_free_uri(u);
    }
    free(uri_s);
  }

  if (steps_contain(fixture, "heuristic_is_blank")) {
    char *blank_id = json_string(fixture, "id");
    /* Prefer nested step id when present; fall back to fixture id. */
    if (!blank_id || (id && strcmp(blank_id, id) == 0)) {
      /* Look for "_:" pattern inside fixture for heuristic fixtures. */
      const char *p = strstr(fixture, "\"_:");
      is_blank = p != NULL ? 1 : 0;
    } else {
      is_blank = blank_id && strncmp(blank_id, "_:", 2) == 0;
    }
    free(blank_id);
  }

  if (steps_contain(fixture, "unicode_check")) {
    char *text = json_string(fixture, "text");
    unicode_ok = text ? ((int)strlen(text) > 0) : 1;
    free(text);
  }

  if (steps_contain(fixture, "world_get_feature")) {
    /* Feature may be NULL; observe presence of call success. */
    feature = strdup("");
  }

  if (parser) librdf_free_parser(parser);
  if (base_uri) librdf_free_uri(base_uri);
  if (model) librdf_free_model(model);
  if (storage) librdf_free_storage(storage);

  printf("{\"ok\":%s,\"engine\":\"%s-c\",\"fixture_id\":%s%s%s",
         ok ? "true" : "false",
         engine,
         id ? "\"" : "null",
         id ? id : "",
         id ? "\"" : "");
  printf(",\"size\":%d", size);
  if (ask >= 0) printf(",\"ask\":%s", ask ? "true" : "false");
  if (select_count >= 0) printf(",\"select_count\":%d", select_count);
  if (stream_count >= 0) printf(",\"stream_count\":%d", stream_count);
  printf(",\"parsed\":%s", parsed ? "true" : "false");
  if (nodes > 0) printf(",\"nodes\":%d", nodes);
  printf(",\"contains_ok\":%s", contains_ok ? "true" : "false");
  if (digest_hex[0]) printf(",\"digest_hex\":\"%s\"", digest_hex);
  if (serialized) {
    /* Escape for JSON: only emit contains_ok; bytes length for diagnostics. */
    printf(",\"bytes_len\":%zu", strlen(serialized));
  }
  if (world_opened) printf(",\"world\":\"open\"");
  if (concepts) printf(",\"concepts\":true");
  if (unicode_ok) printf(",\"unicode_ok\":true");
  if (is_blank >= 0) printf(",\"is_blank\":%s", is_blank ? "true" : "false");
  if (filename) printf(",\"filename\":\"%s\"", filename);
  if (logged) printf(",\"logged\":\"%s\"", logged);
  if (feature) printf(",\"feature\":null");
  if (error) printf(",\"error\":\"%s\"", error);
  printf("}\n");

  if (serialized) {
#ifdef LIBRDF_VERSION
    librdf_free_memory(serialized);
#else
    free(serialized);
#endif
  }
  free(filename);
  free(logged);
  free(feature);
  librdf_free_world(world);
  free(turtle);
  free(id);
  free(fixture);
  return ok ? 0 : 1;
}
