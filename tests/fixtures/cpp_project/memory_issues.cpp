#include <cstdlib>
#include <cstdio>

// Memory leak: allocated but never freed
void memory_leak() {
    int* data = (int*)malloc(100 * sizeof(int));
    data[0] = 42;
    printf("Value: %d\n", data[0]);
    // Missing free(data)
}

// Use after free
void use_after_free() {
    int* ptr = new int(10);
    delete ptr;
    printf("Dangling: %d\n", *ptr);  // Use after free
}

// Double free
void double_free() {
    int* arr = new int[10];
    delete[] arr;
    delete[] arr;  // Double free
}

// Null pointer dereference
void null_deref(int* input) {
    // No null check before use
    printf("Value: %d\n", *input);
}

int main() {
    memory_leak();
    use_after_free();
    double_free();
    null_deref(nullptr);
    return 0;
}
