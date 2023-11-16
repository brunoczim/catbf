#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

struct fast_bfc_interface {
    FILE *in;
    FILE *out;
};

int8_t fast_bfc_put(struct fast_bfc_interface *interface, uint8_t ch);

int16_t fast_bfc_get(struct fast_bfc_interface *interface);

uint8_t *fast_bfc_grow_next(uint8_t *tape_start, uint64_t tape_len);

uint8_t *fast_bfc_grow_prev(uint8_t *tape_start, uint64_t tape_len);

int8_t fast_bfc_main(
    struct fast_bfc_interface *interface,
    uint8_t *tape_start,
    uint64_t tape_len);

int main(int argc, char const *argv[])
{
    int exit_code = 0;

    struct fast_bfc_interface interface;
    uint8_t *tape_start;
    uint64_t tape_len;
    int8_t result;
    interface.in = stdin;
    interface.out = stdout;
    tape_len = 8192;
    tape_start = malloc(sizeof(*tape_start) * tape_len);
    result = fast_bfc_main(&interface, tape_start, tape_len);
    if (result < 0) {
        exit_code = 1;
        perror("stdio");
    }
    free(tape_start);

    return exit_code;
}

int8_t fast_bfc_put(struct fast_bfc_interface *interface, uint8_t ch)
{
    if (fputc(ch, interface->out) < 0) {
        return -1;
    }
    return 0;
}

int16_t fast_bfc_get(struct fast_bfc_interface *interface)
{
    int16_t result = fgetc(interface->in);
    if (feof(interface->in)) {
        return 0;
    }
    if (result < 0) {
        return -1;
    }
    return (1 << 8) | result;
}

uint8_t *fast_bfc_grow_next(uint8_t *tape_start, uint64_t tape_len)
{
    return realloc(tape_start, tape_len + 8192);
}

uint8_t *fast_bfc_grow_prev(uint8_t *tape_start, uint64_t tape_len)
{
    uint8_t *new_start = realloc(tape_start, tape_len + 8192);
    memmove(new_start + 8192, new_start, 8192);
    return new_start;
}
