#include <stdio.h>

int sum(int, int, int);
int fibo(int);

int main () {
    printf("\nCompiled:\n");
    printf("sum(1, 2, 3) = %i\n", sum(1, 2, 3));
    printf("fibo(10) = %i\n", fibo(10));
}
