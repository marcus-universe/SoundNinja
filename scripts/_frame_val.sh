#!/bin/bash
set -u
mount=$(ls -d /tmp/.mount_Sound* 2>/dev/null | head -1)
echo "mount $mount"
bin="$mount/usr/bin/soundninja"
ls -la "$bin"
cat > /tmp/sn_val.c << 'EOF'
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
typedef struct BrotliDecoderStateStruct BrotliDecoderState;
typedef enum { ERR=0, OK=1, MOREIN=2, MOREOUT=3 } R;
BrotliDecoderState* BrotliDecoderCreateInstance(void*, void*, void*);
void BrotliDecoderDestroyInstance(BrotliDecoderState*);
R BrotliDecoderDecompressStream(BrotliDecoderState*, size_t*, const uint8_t**, size_t*, uint8_t**, size_t*);
static int pc(unsigned char c){return (c>='a'&&c<='z')||(c>='A'&&c<='Z')||(c>='0'&&c<='9')||c=='/'||c=='.'||c=='_'||c=='-';}
int main(int argc, char** argv){
  FILE* f=fopen(argv[1],"rb");
  fseek(f,0,SEEK_END); long n=ftell(f); fseek(f,0,SEEK_SET);
  uint8_t* data=malloc(n); fread(data,1,n,f); fclose(f);
  size_t cap=24*1024*1024; uint8_t* out=malloc(cap);
  const char* needle="$value";
  size_t nl=strlen(needle);
  int shown=0;
  for(long i=0;i+16<n;i++){
    if(memcmp(data+i,"/_nuxt/",7)!=0) continue;
    long j=i; while(j<n && pc(data[j])) j++;
    if(j-i<8 || memcmp(data+j-3,".js",3)!=0){ i=j; continue; }
    BrotliDecoderState* st=BrotliDecoderCreateInstance(0,0,0);
    const uint8_t* nin=data+j; size_t ain=(size_t)(n-j); if(ain>12*1024*1024) ain=12*1024*1024;
    uint8_t* nout=out; size_t aout=cap; size_t total=0;
    R r=BrotliDecoderDecompressStream(st,&ain,&nin,&aout,&nout,&total);
    BrotliDecoderDestroyInstance(st);
    if(r!=OK || total<50){ i=j; continue; }
    for(size_t k=0;k+nl<total;k++){
      if(memcmp(out+k,needle,nl)!=0) continue;
      size_t a=k>100?k-100:0; size_t b=k+160<total?k+160:total;
      printf("%.*s @%zu\n", (int)(j-i), data+i, k);
      fwrite(out+a,1,b-a,stdout); fputc('\n',stdout);
      shown++;
      if(shown>=8) return 0;
      k+=nl;
    }
    i=j;
  }
  printf("shown %d\n", shown);
  return 0;
}
EOF
gcc -O2 -o /tmp/sn_val /tmp/sn_val.c -lbrotlidec -lbrotlicommon && /tmp/sn_val "$bin"
