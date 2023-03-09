#ifndef TRIDENT_H
#define TRIDENT_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * C side constructs
 */
typedef struct wrapped_examiner_t {
  void *_0;
} wrapped_examiner_t;

extern void *malloc(uintptr_t layout_size);

extern void free(void *ptr);

extern void turn_on_pump_for_duration(int32_t amount);

/**
 * Initializes an examiner and assigns its pointer to one that has been passed in from C side
 *
 * # Arguments
 *
 * * `_examiner_ptr` - A pointer to the wrapped struct that is really a tuple struct
 * The consumer of this api does not have to work with the inners of this struct outside of the
 * provided api
 */
int32_t initialize_examiner(struct wrapped_examiner_t **_examiner_ptr);

/**
 * Takes an initialized wrapped examiner and a humidity reading and evaluates all rules.
 * It will also call action function (i.e. turn on the pump) if the evaluations deems it
 * necessary.
 * Note that the action function is supplied statically during compile time via the use of extern
 * function.
 *
 * # Arguments
 *
 * * `examiner_ptr` - A pointer to an initialized wrapped examiner.
 * * `humd_reading` - An integer representing the humdity reading in percentage
 */
void handle_humd_input(struct wrapped_examiner_t *examiner_ptr, int32_t humd_reading);

/**
 * Takes an initialized wrapped examiner and frees it.
 * This is technically not needed here because we are not using std and the allocator we are using
 * are the same one used on C side. Therefore technically the freeing of memory can also be done
 * on C side
 *
 * # Arguments
 *
 * * `examiner_ptr` - Apointer to an initialized wrapped examiner.
 */
void free_wrapped_examiner(struct wrapped_examiner_t *examiner_ptr);

extern void turn_on_pump_for_duration(int32_t amount);

#endif /* TRIDENT_H */
