/* Independent unsigned-C arithmetic oracle; compiled to author_vectors.txt.
 * Blackman/Vigna generator formulas and jump constants are public domain:
 * https://prng.di.unimi.it/ (individual *.c reference implementations).
 * PCG: https://www.pcg-random.org/download.html (algorithm/known answers).
 * JSF: https://burtleburtle.net/bob/rand/smallprng.html
 * SFC: PractRand / https://github.com/rust-random/rngs/tree/master/rand_sfc
 * Squares: Widynski, https://arxiv.org/abs/2004.06278
 * Xorshift/XORWOW: Marsaglia, https://www.jstatsoft.org/article/view/v008i14
 * CET/Biski/SplitMix32 rows test the explicitly implemented mathematical
 * recurrence, not a claim that an external canonical specification was found.
 * This harness is test-only, not linked into the Rust implementation.
 */
#include <stdint.h>
#include <stdio.h>
#include <inttypes.h>
static uint32_t r32(uint32_t x,unsigned n){return(x<<n)|(x>>(32-n));}
static uint64_t r64(uint64_t x,unsigned n){return(x<<n)|(x>>(64-n));}
static uint32_t x128(uint32_t*s,int ss){
 uint32_t out=ss?r32(s[1]*5,7)*9:r32(s[0]+s[3],7)+s[0],t=s[1]<<9;
 s[2]^=s[0];s[3]^=s[1];s[1]^=s[2];s[0]^=s[3];s[2]^=t;s[3]=r32(s[3],11);return out;
}
static uint64_t x256(uint64_t*s,int ss){
 uint64_t out=ss?r64(s[1]*5,7)*9:r64(s[0]+s[3],23)+s[0],t=s[1]<<17;
 s[2]^=s[0];s[3]^=s[1];s[1]^=s[2];s[0]^=s[3];s[2]^=t;s[3]=r64(s[3],45);return out;
}
static uint32_t xo64(uint32_t*s){uint32_t a=s[0],b=s[1],out=r32(a*UINT32_C(0x9e3779bb),5)*5;b^=a;s[0]=r32(a,26)^b^(b<<9);s[1]=r32(b,13);return out;}
static uint64_t xo128(uint64_t*s,int ss){
 uint64_t a=s[0],b=s[1],out=ss?r64(a*5,7)*9:r64(a+b,17)+a;b^=a;
 s[0]=r64(a,ss?24:49)^b^(b<<(ss?16:21));s[1]=r64(b,ss?37:28);return out;
}
static void jump(uint64_t*s,int ss,int large){
 static const uint64_t p[2][2][2]={
  {{UINT64_C(0x2bd7a6a6e99c2ddc),UINT64_C(0x0992ccaf6a6fca05)},{UINT64_C(0x360fd5f2cf8d5d99),UINT64_C(0x9c6e6877736c46e3)}},
  {{UINT64_C(0xdf900294d8f554a5),UINT64_C(0x170865df4b3201fc)},{UINT64_C(0xd2a98b26625eee7b),UINT64_C(0xdddf9b1090aa7ac1)}}};
 uint64_t a=0,b=0;for(int i=0;i<2;i++)for(int j=0;j<64;j++){if(p[ss][large][i]&(UINT64_C(1)<<j)){a^=s[0];b^=s[1];}xo128(s,ss);}s[0]=a;s[1]=b;
}
static uint32_t sfc32(uint32_t*s){uint32_t t=s[0]+s[1]+s[3]++;s[0]=s[1]^(s[1]>>9);s[1]=s[2]+(s[2]<<3);s[2]=r32(s[2],21)+t;return t;}
static uint64_t sfc64(uint64_t*s){uint64_t t=s[0]+s[1]+s[3]++;s[0]=s[1]^(s[1]>>11);s[1]=s[2]+(s[2]<<3);s[2]=r64(s[2],24)+t;return t;}
static uint32_t jsf(uint32_t*s){uint32_t e=s[0]-r32(s[1],27);s[0]=s[1]^r32(s[2],17);s[1]=s[2]+s[3];s[2]=s[3]+e;s[3]=e+s[0];return s[3];}
static uint32_t pcg(uint64_t*s){uint64_t old=s[0];s[0]=old*UINT64_C(6364136223846793005)+s[1];uint32_t x=((old>>18)^old)>>27,r=old>>59;return(x>>r)|(x<<((-r)&31));}
static uint32_t xs32(uint32_t*s){uint32_t x=*s;x^=x<<13;x^=x>>17;x^=x<<5;return *s=x;}
static uint32_t xs128(uint32_t*s){uint32_t t=s[3];t^=t<<11;t^=t>>8;uint32_t x=s[0];s[3]=s[2];s[2]=s[1];s[1]=x;return s[0]=t^x^(x>>19);}
static uint32_t xorwow(uint32_t*s){uint32_t t=s[4],a=s[0];for(int i=4;i>0;i--)s[i]=s[i-1];t^=t>>2;t^=t<<1;t^=a^(a<<4);s[0]=t;s[5]+=362437;return t+s[5];}
static uint32_t sm32(uint32_t*s){*s+=UINT32_C(0x9e3779b9);uint64_t z=*s;z=(z^(z>>16))*UINT64_C(0xff51afd7ed558ccd);z=(z^(z>>16))*UINT64_C(0xc4ceb9fe1a85ec53);return z^(z>>16);}
static uint64_t sm64(uint64_t*s){uint64_t z=(*s+=UINT64_C(0x9e3779b97f4a7c15));z=(z^(z>>30))*UINT64_C(0xbf58476d1ce4e5b9);z=(z^(z>>27))*UINT64_C(0x94d049bb133111eb);return z^(z>>31);}
static uint64_t biski(uint64_t*s){uint64_t out=s[1]+s[2],a=s[0],b=s[1],c=s[2];s[0]=a+UINT64_C(0x9999999999999999);s[1]=r64(b,16)+r64(c,40);s[2]=a^b;return out;}
static uint64_t cet_mix(uint64_t x){x^=x>>30;x*=UINT64_C(0xffffffffffffff1b);x^=x>>27;x*=UINT64_C(0x94d049bb133111eb);return x^(x>>31);}
static uint64_t cet64(uint64_t*s){*s+=UINT64_C(0xffffffffffffff43);return cet_mix(*s);}
static uint64_t cet256(uint64_t*s){uint64_t inc=UINT64_C(0xffffffffffffff43);for(int i=0;i<4;i++){uint64_t old=s[i];s[i]+=inc;inc=s[i]<old;}return cet_mix((s[0]^s[3])+r64(s[1],17));}
static uint32_t squares(uint64_t*s){uint64_t y=s[0]++*s[1],z=y+s[1],x=y*y+y;x=r64(x,32);x=x*x+z;x=r64(x,32);x=x*x+y;x=r64(x,32);return(x*x+z)>>32;}
#define ROW32(name,expr) do{uint32_t s[6]={1,2,3,4,5,6};printf("%s",name);for(int n=0;n<16;n++)printf(" %08"PRIx32,(uint32_t)(expr));puts("");}while(0)
#define ROW64(name,expr) do{uint64_t s[4]={1,2,3,4};printf("%s",name);for(int n=0;n<16;n++)printf(" %016"PRIx64,(uint64_t)(expr));puts("");}while(0)
int main(void){
 ROW32("xoshiro128pp",x128(s,0));ROW32("xoshiro128ss",x128(s,1));
 ROW64("xoshiro256pp",x256(s,0));ROW64("xoshiro256ss",x256(s,1));
 ROW32("xoroshiro64ss",xo64(s));ROW64("xoroshiro128pp",xo128(s,0));ROW64("xoroshiro128ss",xo128(s,1));
 ROW32("sfc32",sfc32(s));ROW64("sfc64",sfc64(s));ROW32("jsf32",jsf(s));
 ROW32("xorshift32",xs32(s));ROW32("xorshift128",xs128(s));ROW32("xorwow",xorwow(s));
 ROW32("splitmix32",sm32(s));ROW64("splitmix64",sm64(s));ROW64("biski64",biski(s));
 ROW64("cet64",cet64(s));ROW64("cet256",cet256(s));
 uint64_t carry[4]={UINT64_MAX,UINT64_MAX,UINT64_MAX,UINT64_MAX};printf("cet256_carry");for(int i=0;i<16;i++)printf(" %016"PRIx64,cet256(carry));puts("");
 uint64_t p[2]={0,109};pcg(p);p[0]+=42;pcg(p);printf("pcg32");for(int i=0;i<16;i++)printf(" %08"PRIx32,pcg(p));puts("");
 uint64_t q[2]={0,UINT64_C(0xc8e4fd154ce32f6d)};printf("squares32");for(int i=0;i<16;i++)printf(" %08"PRIx32,squares(q));puts("");
 for(int ss=0;ss<2;ss++)for(int large=0;large<2;large++){uint64_t s[2]={1,2};jump(s,ss,large);printf("xoroshiro128%s_%s",ss?"ss":"pp",large?"long_jump":"jump");for(int i=0;i<16;i++)printf(" %016"PRIx64,xo128(s,ss));puts("");}
 return 0;
}
