#include <cstdio>
#include <cstring>

// Format string vulnerability
void log_message(const char* user_input) {
    printf(user_input);  // User-controlled format string
}

// Uninitialized variable
int compute() {
    int result;
    // result is never initialized
    return result + 1;
}

// Integer overflow
void integer_overflow() {
    int max = 2147483647;
    int overflowed = max + 1;  // Signed integer overflow (UB)
    printf("Result: %d\n", overflowed);
}

// Unsafe use of system()
void run_command(const char* user_input) {
    char cmd[256];
    sprintf(cmd, "echo %s", user_input);  // Command injection
    system(cmd);
}

int main() {
    log_message("%s%s%s%s");
    printf("Computed: %d\n", compute());
    integer_overflow();
    run_command("hello; rm -rf /");
    return 0;
}
