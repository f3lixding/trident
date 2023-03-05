#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

extern void *malloc(uint32_t layout_size);

extern void free(void *ptr);

void test_func_for_bindgen(void);

extern void turn_on_pump_for_duration(int32_t amount);
