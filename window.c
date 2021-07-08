#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <unistd.h>

#include "window.h"

int read_window_size(int socket, window_size_t *size)
{
    write(socket, "SIZE\n", 5);

    char buffer[20];

    int read_bytes = read(socket, buffer, 20);

    buffer[read_bytes] = 0;

    char *x = NULL;
    char *y = NULL;
    int space_count = 0;

    for (char *i = buffer; i < buffer + read_bytes; i++)
    {
        if (*i == ' ' || *i == '\n')
        {
            *i = '\0';
            space_count++;
            if (space_count == 1)
            {
                x = i + 1;
            }
            else if (space_count == 2)
            {
                y = i + 1;
            }
        }
    }

    if (x == NULL || y == NULL)
    {
        return 0;
    }

    (*size).x_width = strtol(x, NULL, 10);

    (*size).y_height = strtol(y, NULL, 10);

    return 1;
}

int fill_screen(int socket_fd, window_size_t *size, color_t *color)
{

    char color_buf[9];
    print_color(color_buf, color);

    char *px = "PX ";

    char format[64];
    sprintf(format, "PX %%d %%d %s\n", color_buf);

    // Overallocate to minimize need for reallocations
    int buffer_size = (size->x_width * size->y_height) * (3 + 8 + 8 + 10);
    char *buffer = malloc(buffer_size);

    char *buffer_index = buffer;
    for (int x = 0; x < size->x_width; x++)
    {
        for (int y = 0; y < size->y_height; y++)
        {
            char local_buffer[64];
            int len = sprintf(local_buffer, format, x, y);
            while (buffer_index + len + 1 > buffer + buffer_size)
            {
                // Save the current buffer index offset
                int index_offset = buffer_index - buffer;

                buffer_size *= 2;
                buffer = realloc(buffer, buffer_size);

                // Reassign buffer index offset correctly
                buffer_index = buffer + index_offset;
            }

            memcpy(buffer_index, local_buffer, len);
            buffer_index += len;
        }
    }

    write(socket_fd, buffer, buffer_index - buffer);

    free(buffer);
}

void shuffle(position_t *array, int n)
{
    if (n > 1)
    {
        size_t i;
        for (i = 0; i < n - 1; i++)
        {
            size_t j = i + rand() / (RAND_MAX / (n - i) + 1);
            position_t t = array[j];
            array[j] = array[i];
            array[i] = t;
        }
    }
}

int fill_screen_noisy(int socket_fd, window_size_t *size, color_t *color)
{
    int length = size->x_width * size->y_height;
    position_t *positions = malloc(sizeof(position_t) * length);

    for (int x = 0; x < size->x_width; x++)
    {
        for (int y = 0; y < size->y_height; y++)
        {
            position_t *position = &positions[(x * size->y_height) + y];
            position->x = x;
            position->y = y;
        }
    }

    shuffle(positions, length);

    char color_buf[9];
    print_color(color_buf, color);

    char *px = "PX ";

    char format[64];
    sprintf(format, "PX %%d %%d %s\n", color_buf);

    // Overallocate to minimize need for reallocations
    int buffer_size = length * (3 + 8 + 8 + 10);
    char *buffer = malloc(buffer_size);

    char *buffer_index = buffer;
    for (int i = 0; i < length; i++)
    {
        char local_buffer[64];
        int len = sprintf(local_buffer, format, positions[i].x, positions[i].y);
        while (buffer_index + len + 1 > buffer + buffer_size)
        {
            // Save the current buffer index offset
            int index_offset = buffer_index - buffer;

            buffer_size *= 2;
            buffer = realloc(buffer, buffer_size);

            // Reassign buffer index offset correctly
            buffer_index = buffer + index_offset;
        }

        memcpy(buffer_index, local_buffer, len);
        buffer_index += len;
    }

    write(socket_fd, buffer, buffer_index - buffer);

    free(buffer);
    free(positions);
}