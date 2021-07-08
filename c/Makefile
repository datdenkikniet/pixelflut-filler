FILES := main.c color.h color.c window.c window.h

all: $(FILES) bin
	gcc -o bin/pixeflut-filler $(FILES)

debug: $(FILES) bin
	gcc -o bin/pixeflut-filler-debug $(FILES)

bin:
	mkdir -p bin/