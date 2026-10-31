#!/bin/bash
set -u
cat > /tmp/sn_unbr2.c << 'EOF'
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
BrotliDecoderState* BrotliDecoderCreateInstance(void*, void*, void*);
void BrotliDecoderDestroyInstance(BrotliDecoderState*);
BrotliDecoderResult BrotliDecoderDecompressStream(
    BrotliDecoderState*, size_t*, const uint8_t**, size_t*, uint8_t**, size_t*);

static int is_path_char(unsigned char c) {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
         (c >= '0' && c <= '9') || c == '/' || c == '.' || c == '_' || c == '-';
}

static BrotliDecoderResult dec_at(uint8_t* data, long n, long off, uint8_t* out, size_t cap, size_t* total) {
  BrotliDecoderState* st = BrotliDecoderCreateInstance(NULL, NULL, NULL);
  const uint8_t* next_in = data + off;
  size_t avail_in = (size_t)(n - off);
  if (avail_in > 12 * 1024 * 1024) avail_in = 12 * 1024 * 1024;
  uint8_t* next_out = out;
  size_t avail_out = cap;
  *total = 0;
  BrotliDecoderResult r = BrotliDecoderDecompressStream(st, &avail_in, &next_in, &avail_out, &next_out, total);
  BrotliDecoderDestroyInstance(st);
  return r;
}

static void show(uint8_t* out, size_t total, const char* needle, const char* label) {
  size_t nlen = strlen(needle);
  int hits = 0;
  for (size_t k = 0; k + nlen < total && hits < 4; k++) {
    if (memcmp(out + k, needle, nlen) != 0) continue;
    size_t a = k > 180 ? k - 180 : 0;
    size_t b = k + 280 < total ? k + 280 : total;
    printf("--- %s @%zu ---\n", label, k);
    fwrite(out + a, 1, b - a, stdout);
    fputc('\n', stdout);
    hits++;
    k += nlen;
  }
}

int main(void) {
  const char* path = "/home/steamos/.local/share/soundninja/extract-081/squashfs-root/usr/bin/soundninja";
  FILE* f = fopen(path, "rb");
  fseek(f, 0, SEEK_END);
  long n = ftell(f);
  fseek(f, 0, SEEK_SET);
  uint8_t* data = malloc(n);
  fread(data, 1, n, f);
  fclose(f);
  size_t cap = 20 * 1024 * 1024;
  uint8_t* out = malloc(cap);
  int assets = 0, ok = 0;
  for (long i = 0; i + 16 < n; i++) {
    if (memcmp(data + i, "/_nuxt/", 7) != 0) continue;
    long j = i;
    while (j < n && is_path_char(data[j])) j++;
    long name_len = j - i;
    if (name_len < 8 || !(name_len >= 3 && memcmp(data + j - 3, ".js", 3) == 0)) {
      i = j > i ? j : i + 1;
      continue;
    }
    assets++;
    size_t total = 0;
    BrotliDecoderResult r = BROTLI_DECODER_RESULT_ERROR;
    long used = j;
    for (int delta = 0; delta < 8; delta++) {
      r = dec_at(data, n, j + delta, out, cap, &total);
      if (r == BROTLI_DECODER_RESULT_SUCCESS && total > 50) { used = j + delta; break; }
    }
    char name[96];
    int nl = (int)(name_len < 90 ? name_len : 90);
    memcpy(name, data + i, nl);
    name[nl] = 0;
    if (r != BROTLI_DECODER_RESULT_SUCCESS || total < 50) {
      printf("FAIL %s r=%d\n", name, (int)r);
      i = j;
      continue;
    }
    ok++;
    int has_ins = 0, has_fail = 0, has_save = 0;
    for (size_t k = 0; k + 18 < total; k++) {
      if (!has_ins && memcmp(out + k, "INSERT INTO sounds", 18) == 0) has_ins = 1;
      if (!has_fail && memcmp(out + k, "failed_", 7) == 0) has_fail = 1;
      if (!has_save && memcmp(out + k, "saveConfig", 10) == 0) has_save = 1;
    }
    printf("OK %s bytes=%zu ins=%d fail=%d save=%d off=%ld\n", name, total, has_ins, has_fail, has_save, used - j);
    if (has_ins) show(out, total, "INSERT INTO sounds", name);
    if (has_fail) show(out, total, "failed_", name);
    i = j;
  }
  printf("assets=%d ok=%d\n", assets, ok);
  free(out);
  free(data);
  return 0;
}
EOF
gcc -O2 -o /tmp/sn_unbr2 /tmp/sn_unbr2.c -lbrotlidec -lbrotlicommon && /tmp/sn_unbr2
echo "=== db text search ==="
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
sqlite3 "$db" "SELECT key FROM settings WHERE value LIKE '%fail%' OR key LIKE '%fail%';"
sqlite3 "$db.bak" "SELECT key FROM settings WHERE value LIKE '%fail%' OR key LIKE '%fail%';"
