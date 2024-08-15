#include <stdio.h>

void user_main();

int main () {
    printf("Compiled C:\n");

    printf("fibo(10) = ");
    fflush(stdout);

    user_main();

    printf("\n");
}
