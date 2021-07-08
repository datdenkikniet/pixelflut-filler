#include <fcntl.h>
#include <string.h>
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>

#include "color.h"

int print_color(char *destination, color_t *color)
{
    return sprintf(destination, "%02X%02X%02X%02X", color->r, color->g, color->b, color->a);
}

color_t generate_random_color()
{
    int random = open("/dev/urandom", O_RDONLY);

    uint8_t random_data[4];
    read(random, random_data, 4);

    color_t final_color;
    final_color.r = random_data[0];
    final_color.g = random_data[1];
    final_color.b = random_data[2];
    final_color.a = random_data[3];

    return final_color;
}

int from_hex_digit(char digit)
{
    char *endp;
    int result = strtol(&digit, &endp, 16);

    if (*endp != '\0')
    {
        return -1;
    }
    else
    {
        return result;
    }
}

int parse_color(color_t *result, char *input)
{
    int len = strlen(input);
    if (len != 6 && len != 8)
    {
        return 1;
    }
    for (int i = 0; i < len; i++)
    {
        if (from_hex_digit(input[i]) == -1)
        {
            return 2;
        }
    }

    result->r = (from_hex_digit(input[0]) * 16) + from_hex_digit(input[1]);
    result->g = (from_hex_digit(input[2]) * 16) + from_hex_digit(input[3]);
    result->b = (from_hex_digit(input[4]) * 16) + from_hex_digit(input[5]);
    if (len > 6)
    {
        result->a = (from_hex_digit(input[6]) * 16) + from_hex_digit(input[7]);
    }
    else
    {
        result->a = 0xFF;
    }
}
