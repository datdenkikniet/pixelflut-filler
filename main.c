#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>

#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

typedef struct
{
	int x_width;
	int y_height;
} window_size;

int read_size(int socket, window_size *size)
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

int setup_socket(char *remote)
{
	int sockfd = socket(AF_INET, SOCK_STREAM, 0);

	if (sockfd == -1)
	{
		return -1;
	}

	struct sockaddr_in server_addr;

	server_addr.sin_family = AF_INET;
	int valid_ip = inet_aton(remote, &server_addr.sin_addr);

	if (!valid_ip)
	{
		return -2;
	}

	server_addr.sin_port = htons(1337);

	int connected = connect(sockfd, (struct sockaddr *)&server_addr, sizeof(server_addr));

	if (connected == -1)
	{
		return -3;
	}

	return sockfd;
}

int generate_color(char *destination)
{
	int random = open("/dev/urandom", O_RDONLY);

	uint8_t color[4];
	read(random, color, 4);

	return sprintf(destination, "%02X%02X%02X%02X", color[0], color[1], color[2], color[3]);
}

int main(int argc, char *argv[])
{
	int sockfd;
	if (argc > 1)
	{
		printf("Connecting to %s\n", argv[1]);
		sockfd = setup_socket(argv[1]);
	}
	else
	{
		printf("Connecting to 127.0.0.1\n");
		sockfd = setup_socket("127.0.0.1");
	}

	if (sockfd == -1)
	{
		printf("Could not create socket.\n");
		return 1;
	}
	else if (sockfd == -2)
	{
		printf("Invalid remote address\n");
		return 2;
	}
	else if (sockfd == -3)
	{
		printf("Could not connect to remote\n");
		return 3;
	}

	char color_buf[9];
	if (argc > 2)
	{
		char *carg = argv[2];
		int len = strlen(carg);
		if (len != 6 && len != 8)
		{
			printf("Invalid color (6 or 8 characters).\n");
			return 4;
		}
		for (int i = 0; i < len; i++)
		{
			if ((carg[i] < 'A' && carg[i] > 'F') && (carg[i] < '0' && carg[i] > '9'))
			{
				printf("Invalid color (must be upper hex).\n");
				return 5;
			}
		}

		sprintf(color_buf, "%s", carg);
	}
	else
	{
		generate_color(color_buf);
	}
	window_size size;
	read_size(sockfd, &size);

	printf("Detected a window with dimensions %d, %d\n", size.x_width, size.y_height);
	printf("Filling it with color %s\n", color_buf);

	char *px = "PX ";

	char format[64];
	sprintf(format, "PX %%d %%d %s\n", color_buf);

	// Overallocate to minimize need for reallocations
	int buffer_size = (size.x_width * size.y_height) * (3 + 8 + 8 + 10);
	char *buffer = malloc(buffer_size);

	char *buffer_index = buffer;
	for (int x = 0; x < size.x_width; x++)
	{
		for (int y = 0; y < size.y_height; y++)
		{
			char local_buffer[64];
			int len = sprintf(local_buffer, format, x, y);
			while (buffer_index + len + 1 > buffer + buffer_size)
			{
				printf("Reallocating.");
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

	write(sockfd, buffer, buffer_index - buffer);

	close(sockfd);

	return 0;
}
