#ifndef COLOR_H
#define COLOR_H

#include <stdint.h>

typedef struct
{
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
} color_t;

int print_color(char *, color_t *);
color_t generate_random_color();
int parse_color(color_t *, char *);

#endif