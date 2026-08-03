/* Variadic Redland-shaped librdf_log forwarding into Rust librdf_log_simple. */
#include <stdarg.h>
#include <stdio.h>

struct librdf_world_s;
typedef struct librdf_world_s librdf_world;

void librdf_log_simple(librdf_world *world, int code, int level, int facility,
                       void *locator, const char *message);

void librdf_log(librdf_world *world, int code, int level, int facility,
                void *locator, const char *message, ...) {
  char buffer[4096];
  va_list args;
  va_start(args, message);
  if (message == NULL) {
    buffer[0] = '\0';
  } else {
    vsnprintf(buffer, sizeof(buffer), message, args);
  }
  va_end(args);
  librdf_log_simple(world, code, level, facility, locator, buffer);
}
