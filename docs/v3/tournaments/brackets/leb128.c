#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>

const uint8_t CONTINUE_BIT = 1 << 7;

void encode(uint64_t n, unsigned char* buf, int len) {
    int bytes_written = 0;

    while (true) {
        unsigned char byte = (n & 255) & ~CONTINUE_BIT;

        // Move the the next group of 7-bit.
        n >>= 7;

        // If there are more bytes following we need to set the continue bit.
        if (n != 0) {
            byte |= CONTINUE_BIT;
        }

        // Prevent accidental buffer overflows. If you use something less primitive
        // than raw pointers, this isn't necessary.
        if (bytes_written >= len) {
            return;
        }

        // Write the byte into the buffer.
        buf[bytes_written] = byte;
        bytes_written += 1;

        // Continue with next byte if required.
        if (n == 0) {
            return;
        }
    }
}

// Note that we're not doing any error handling here. Instead we always return 0 when an error
// occurs.
uint64_t decode(unsigned char* buf, int len) {
    uint64_t n = 0;
    int shift = 0;
    int bytes_read = 0;

    while (true) {
        // EOF
        if (bytes_read >= len) {
            return 0;
        }

        // Overflow
        if (shift >= 63) {
            return 0;
        }

        // Read a byte from the buffer.
        unsigned char byte = buf[bytes_read];
        bytes_read += 1;

        // Remove the continue bit, then add the byte.
        n += (byte & ~CONTINUE_BIT) << shift;

        // If the continue bit is 0, the integer has ended.
        if ((byte & CONTINUE_BIT) == 0) {
            return n;
        }

        // Move the next group of 7-bit.
        shift += 7;
    }
}

int main() {
    unsigned char buf[] = {0, 0};
    encode(300, &buf[0], 2);

    for (int i = 0; i < 2; i++) {
        // Should print 172, 2.
        printf("%u,", buf[i]);
    }
    printf("\n");

    uint64_t n = decode(&buf[0], 2);
    // Should print 300.
    printf("%d\n", n);
}
