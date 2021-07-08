#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <getopt.h>

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
	int flags, opt;

	char *remote = NULL;
	char *color_arg = NULL;
	int noisy = 0;

	while ((opt = getopt(argc, argv, "hnr:c:")) != -1)
	{
		switch (opt)
		{
		case 'n':
			noisy = 1;
			break;
		case 'r':
			remote = optarg;
			break;
		case 'c':
			color_arg = optarg;
			break;
		case 'h':
		default:
			printf("Usage: %s [options]\n", argv[0]);
			printf("Options:\n");
			printf("  -r <addr>  The host to connect to. (Default: 127.0.0.1)\n");
			printf("  -n              Enable noisy fill.\n");
			printf("  -c <rrggbb(aa)> The color to fill with in hex, alpha is optional.\n");
			printf("                  (Default: random value)\n");
			printf("  -h              Show this help menu and exit.\n");
			exit(opt != 'h');
		}
	}

	int sockfd;
	if (remote != NULL)
	{
		printf("Attempting to connect to %s\n", remote);
		sockfd = setup_socket(remote);
	}
	else
	{
		printf("Connecting to 127.0.0.1\n");
		sockfd = setup_socket("127.0.0.1");
	}

	if (sockfd == -1)
	{
		printf("Could not create socket.\n");
		exit(2);
	}
	else if (sockfd == -2)
	{
		printf("Invalid remote address\n");
		exit(3);
	}
	else if (sockfd == -3)
	{
		printf("Could not connect to remote\n");
		exit(4);
	}

	char color_buf[9];
	color_t color;
	if (color_arg != NULL)
	{
		int parsed = parse_color(&color, color_arg);

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

	fill_window(sockfd, &window, &color, noisy);

	close(sockfd);

	return 0;
}
