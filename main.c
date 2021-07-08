#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>

#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#include "color.h"
#include "window.h"

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
	color_t color;
	if (argc > 2)
	{
		int parsed = parse_color(&color, argv[2]);

		if (parsed == 1)
		{
			printf("Invalid color (must be 6 or 8 characters).\n");
			return 4;
		}
		else if (parsed == 2)
		{
			printf("Invalid color (must be hex).\n");
			return 5;
		}
	}
	else
	{
		color = generate_random_color();
	}

	print_color(color_buf, &color);

	window_size_t window;
	read_window_size(sockfd, &window);

	printf("Detected a window with dimensions x: %d, y: %d\n", window.x_width, window.y_height);
	printf("Filling it with color %s\n", color_buf);

	fill_screen_noisy(sockfd, &window, &color);

	close(sockfd);

	return 0;
}
