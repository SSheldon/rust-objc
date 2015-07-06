#pragma once

#include <stddef.h>

size_t tests_count();

const char *test_name(size_t i, size_t *len);

void run_test(size_t i);
