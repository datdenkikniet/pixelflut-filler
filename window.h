#ifndef WINDOW_H
#define WINDOW_H

#include "color.h"

typedef struct
{
    int x_width;
    int y_height;
} window_size_t;

typedef struct
{
    int x;
    int y;
} position_t;

int fill_screen(int, window_size_t *, color_t *);
int fill_screen_noisy(int, window_size_t *, color_t *);

int read_window_size(int, window_size_t *);

#endif