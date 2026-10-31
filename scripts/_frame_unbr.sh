#!/bin/bash
set -u
nm -D /usr/lib/libbrotlidec.so | grep -E "BrotliDecoder(Create|Decompress|Destroy)" | head
cat > /tmp/sn_unbr.c << 'EOF'
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

typedef struct BrotliDecoderStateStruct BrotliDecoderState;
typedef enum {
  BROTLI_DECODER_RESULT_ERROR = 0,
  BROTLI_DECODER_RESULT_SUCCESS = 1,
  BROTLI_DECODER_RESULT_NEEDS_MORE_INPUT = 2,
  BROTLI_DECODER_RESULT_NEEDS_MORE_OUTPUT = 3
} BrotliDecoderResult;

BrotliDecoderState* BrotliDecoderCreateInstance(void* a, void* b, void* c);
void BrotliDecoderDestroyInstance(BrotliDecoderState* s);
BrotliDecoderResult BrotliDecoderDecompressStream(
    BrotliDecoderState* s,
    size_t* available_in, const uint8_t** next_in,
    size_t* available_out, uint8_t** next_out,
    size_t* total_out);

static int is_path_char(unsigned char c) {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
         (c >= '0' && c <= '9') || c == '/' || c == '.' || c == '_' || c == '-';
}

int main(void) {
  const char* path = "/home/steamos/.local/share/soundninja/extract-081/squashfs-root/usr/bin/soundninja";
  FILE* f = fopen(path, "rb");
  if (!f) { perror("open"); return 1; }
  fseek(f, 0, SEEK_END);
  long n = ftell(f);
  fseek(f, 0, SEEK_SET);
  uint8_t* data = malloc(n);
  if (fread(data, 1, n, f) != (size_t)n) { perror("read"); return 1; }
  fclose(f);
  const char* needle = "failed_sounds_path";
  int found = 0;
  for (long i = 0; i + 16 < n; i++) {
    if (memcmp(data + i, "/_nuxt/", 7) != 0 && memcmp(data + i, "/index.html", 11) != 0)
      continue;
    long j = i;
    while (j < n && is_path_char(data[j])) j++;
    if (j <= i + 8) continue;
    /* only js/css/html/json */
    long name_len = j - i;
    int interesting = 0;
    if (name_len >= 3 && memcmp(data + j - 3, ".js", 3) == 0) interesting = 1;
    if (name_len >= 5 && memcmp(data + j - 5, ".html", 5) == 0) interesting = 1;
    if (name_len >= 5 && memcmp(data + j - 5, ".json", 5) == 0) interesting = 1;
    if (!interesting) { i = j; continue; }
    BrotliDecoderState* st = BrotliDecoderCreateInstance(NULL, NULL, NULL);
    if (!st) continue;
    const uint8_t* next_in = data + j;
    size_t avail_in = (size_t)(n - j);
    if (avail_in > 8 * 1024 * 1024) avail_in = 8 * 1024 * 1024;
    size_t cap = 16 * 1024 * 1024;
    uint8_t* out = malloc(cap);
    uint8_t* next_out = out;
    size_t avail_out = cap;
    size_t total = 0;
    BrotliDecoderResult r = BrotliDecoderDecompressStream(st, &avail_in, &next_in, &avail_out, &next_out, &total);
    BrotliDecoderDestroyInstance(st);
    if (r == BROTLI_DECODER_RESULT_SUCCESS && total > 20) {
      uint8_t* hit = NULL;
      for (size_t k = 0; k + 18 < total; k++) {
        if (memcmp(out + k, needle, 18) == 0) { hit = out + k; break; }
      }
      fwrite(data + i, 1, (size_t)(j - i), stdout);
      printf(" dec=%zu %s\n", total, hit ? "HIT" : "miss");
      if (hit) {
        size_t off = (size_t)(hit - out);
        size_t a = off > 220 ? off - 220 : 0;
        size_t b = off + 420 < total ? off + 420 : total;
        fwrite(out + a, 1, b - a, stdout);
        fputc('\n', stdout);
        found++;
      }
    }
    free(out);
    i = j;
    if (found >= 6) break;
  }
  printf("done found=%d\n", found);
  free(data);
  return 0;
}
EOF
gcc -O2 -o /tmp/sn_unbr /tmp/sn_unbr.c -lbrotlidec -lbrotlicommon && /tmp/sn_unbr
