#include <cstring>
#include <cstdio>

// Buffer overflow: writing beyond array bounds
void unsafe_copy(const char* input) {
    char buffer[10];
    strcpy(buffer, input);  // No bounds checking
    printf("Copied: %s\n", buffer);
}

// Stack buffer overflow with gets
void read_input() {
    char buf[64];
    printf("Enter text: ");
    gets(buf);  // Dangerous: no size limit
}

int main() {
    const char* long_string = "This string is way too long for the buffer";
    unsafe_copy(long_string);
    read_input();
    return 0;
}
