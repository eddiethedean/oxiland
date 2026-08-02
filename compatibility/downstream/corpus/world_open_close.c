/*
 * Frozen 0.11 C corpus: world open/close lifecycle.
 * Compiles against system Redland and Oxiland librdf.h under -Werror.
 */

#include <stdio.h>
#include <stdlib.h>

#include <librdf.h>

int main(void) {
  librdf_world *world = librdf_new_world();
  if (!world) {
    fprintf(stderr, "world_open_close: librdf_new_world failed\n");
    return 1;
  }
  librdf_world_open(world);
  librdf_free_world(world);
  puts("world_open_close ok");
  return 0;
}
