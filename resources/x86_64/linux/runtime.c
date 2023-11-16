#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define TAPE_CHUNK_SIZE 8192

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
    tape_len = TAPE_CHUNK_SIZE;
    tape_start = calloc(sizeof(*tape_start), tape_len);
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
    uint8_t *new_start = realloc(tape_start, tape_len + TAPE_CHUNK_SIZE);
    memset(new_start + tape_len, 0, TAPE_CHUNK_SIZE);
    return new_start;
}

uint8_t *fast_bfc_grow_prev(uint8_t *tape_start, uint64_t tape_len)
{
    uint8_t *new_start = realloc(tape_start, tape_len + TAPE_CHUNK_SIZE);
    memmove(new_start + TAPE_CHUNK_SIZE, new_start, TAPE_CHUNK_SIZE);
    memset(new_start, 0, TAPE_CHUNK_SIZE);
    return new_start;
}
