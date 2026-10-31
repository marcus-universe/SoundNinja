#!/bin/bash
set -u
python3 - << 'PY'
p="/home/steamos/.local/share/soundninja/extract-081/squashfs-root/usr/bin/soundninja"
data=open(p,"rb").read()
for s in [b"global_index", b"INSERT INTO", b"failed_", b"saveConfig", b"no such column", b"sounds_path", b"looping"]:
    print(s, data.find(s))
PY
# Also scan decompressed for global_index by reusing the binary tool output via a tiny extra C? 
# Quick: python ctypes streaming is slow. Use the C binary idea inline one more time, shorter.
cat > /tmp/sn_mark.c << 'EOF'
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
int main(){
  FILE* f=fopen("/home/steamos/.local/share/soundninja/extract-081/squashfs-root/usr/bin/soundninja","rb");
  fseek(f,0,SEEK_END); long n=ftell(f); fseek(f,0,SEEK_SET);
  uint8_t* data=malloc(n); fread(data,1,n,f); fclose(f);
  size_t cap=20*1024*1024; uint8_t* out=malloc(cap);
  const char* marks[]={"global_index","INSERT INTO","PRAGMA","sound_id","batchInsert","withProjectDb",0};
  for(long i=0;i+16<n;i++){
    if(memcmp(data+i,"/_nuxt/",7)!=0) continue;
    long j=i; while(j<n && pc(data[j])) j++;
    if(j-i<8 || memcmp(data+j-3,".js",3)!=0){ i=j; continue; }
    BrotliDecoderState* st=BrotliDecoderCreateInstance(0,0,0);
    const uint8_t* nin=data+j; size_t ain=(size_t)(n-j); if(ain>12*1024*1024) ain=12*1024*1024;
    uint8_t* nout=out; size_t aout=cap; size_t total=0;
    R r=BrotliDecoderDecompressStream(st,&ain,&nin,&aout,&nout,&total);
    BrotliDecoderDestroyInstance(st);
    printf("%.*s r=%d total=%zu", (int)(j-i), data+i, (int)r, total);
    for(int m=0; marks[m]; m++){
      size_t L=strlen(marks[m]); int hit=0;
      for(size_t k=0;k+L<total;k++) if(memcmp(out+k,marks[m],L)==0){hit=1;break;}
      if(hit) printf(" [%s]", marks[m]);
    }
    /* first 40 bytes of output as text */
    printf(" head=");
    for(size_t k=0;k<40 && k<total;k++){ unsigned char c=out[k]; putchar((c>=32&&c<127)?c:'.'); }
    printf("\n");
    i=j;
  }
  return 0;
}
EOF
gcc -O2 -o /tmp/sn_mark /tmp/sn_mark.c -lbrotlidec -lbrotlicommon && /tmp/sn_mark
