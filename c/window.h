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
    color_t color;

} position_t;

int fill_window(int socket_fd, window_size_t *size, color_t *color, int noisy);

int read_window_size(int socked_fd, window_size_t *window_size);

#endif