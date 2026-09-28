// The game core, built from gorilla-rust's src/ios.rs. Call everything on
// the main thread.
#ifndef GORILLA_H
#define GORILLA_H

#include <stdbool.h>
#include <stdint.h>

bool gr_start(uint64_t seed, bool muted);
bool gr_step(void);
const uint8_t *gr_frame(uint32_t *width, uint32_t *height);
void gr_push_key(uint32_t c);
void gr_set_muted(bool muted);
void gr_reopen_audio(void);
const char *gr_version(void);
const char *gr_last_error(void);

#endif
