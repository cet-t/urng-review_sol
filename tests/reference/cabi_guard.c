/* Windows C caller for the actual exported ABI, with inaccessible guard pages.
 * Generate cabi_generators.inc from the Rust declarations with verify_cabi.py.
 * This checks writes at a page boundary, independent of Rust slice canaries.
 * Only the fixed DLL is executed: the 1.0.0 Ssx2 memory corruption is not run.
 */
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct { const char *name; unsigned seed_bits, word_bits; } Generator;
static const Generator generators[] = {
#include "cabi_generators.inc"
};

static FARPROC load(HMODULE dll, const char *prefix, const char *suffix) {
    char name[128];
    snprintf(name, sizeof name, "%s_%s", prefix, suffix);
    FARPROC p = GetProcAddress(dll, name);
    if (!p) { fprintf(stderr, "Missing export: %s\n", name); exit(2); }
    return p;
}

int main(int argc, char **argv) {
    if (argc != 2) return 2;
    HMODULE dll = LoadLibraryA(argv[1]);
    if (!dll) { fprintf(stderr, "LoadLibrary failed: %lu\n", GetLastError()); return 2; }
    SYSTEM_INFO info;
    GetSystemInfo(&info);
    const size_t page = info.dwPageSize;
    unsigned char *pages = VirtualAlloc(NULL, page * 3, MEM_RESERVE | MEM_COMMIT, PAGE_NOACCESS);
    DWORD old_protect;
    if (!pages || !VirtualProtect(pages + page, page, PAGE_READWRITE, &old_protect)) return 2;
    unsigned checked = 0, unsupported = 0;
    for (size_t i = 0; i < sizeof generators / sizeof generators[0]; ++i) {
        const Generator g = generators[i];
        FARPROC construct = load(dll, g.name, "new");
        void *state = g.seed_bits == 32
            ? ((void *(*)(uint32_t))construct)(123)
            : ((void *(*)(uint64_t))construct)(123);
        if (!state) { ++unsupported; continue; }
        void (*free_rng)(void *) = (void (*)(void *))load(dll, g.name, "free");
        void (*fill)(void *, void *, size_t) = (void (*)(void *, void *, size_t))
            load(dll, g.name, g.word_bits == 32 ? "next_u32s" : "next_u64s");
        const size_t width = g.word_bits / 8;
        const size_t lengths[] = {0, 1, 7, 8, 15, 16, 17, page / width - 1, page / width};
        fill(NULL, NULL, 0);
        for (size_t j = 0; j < sizeof lengths / sizeof lengths[0]; ++j) {
            const size_t bytes = lengths[j] * width;
            unsigned char *out = pages + page * 2 - bytes;
            memset(pages + page, 0xa5, page);
            fill(state, out, lengths[j]);
            for (unsigned char *p = pages + page; p < out; ++p) {
                if (*p != 0xa5) { fprintf(stderr, "%s: prefix overwrite\n", g.name); return 1; }
            }
            /* The final element abuts PAGE_NOACCESS: any overrun faults. */
        }
        free_rng(state);
        ++checked;
    }
    VirtualFree(pages, 0, MEM_RELEASE);
    FreeLibrary(dll);
    printf("C ABI guard pages: %u generators passed, %u unsupported CPUs\n", checked, unsupported);
    return 0;
}
