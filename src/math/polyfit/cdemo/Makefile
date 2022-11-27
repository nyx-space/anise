it:
	gcc -Wall -shared -o libhrmint.so hrmint.c -fPIC -g
	gcc -Wall main.c -o main -lm -L. -lhrmint
	LD_LIBRARY_PATH=. ./main