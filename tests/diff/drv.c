#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "bindings/default/qsp_default.h"
static void out(const QSP_CHAR *s){ for(;*s;++s){ if(*s<128) putchar(*s); else printf("\\u%04x",(unsigned)*s);} }
static void msg(const QSP_CHAR *s){ printf("MSG:"); out(s); putchar('\n'); }
static void sw(int t,int on){printf("WIN:%d:%d\n",t,on);}
int main(int argc,char**argv){
  FILE*f=fopen(argv[1],"rb"); fseek(f,0,SEEK_END); long n=ftell(f); fseek(f,0,SEEK_SET);
  char*b=malloc(n+4); fread(b,1,n,f); fclose(f);
  QSPInit(); QSPSetCallBack(QSP_CALL_SHOWMSGSTR,(QSP_CALLBACK)msg);QSPSetCallBack(QSP_CALL_SHOWWINDOW,(QSP_CALLBACK)sw);
  static QSP_CHAR nm[]={'t','.','q','s','p',0};
  if(!QSPLoadGameWorldFromData(b,(int)n,nm)){printf("LOADFAIL\n");return 0;}
  int ok=QSPRestartGame(1);
  if(!ok){int e,a,l;QSP_CHAR*loc;QSPGetLastErrorData(&e,&loc,&a,&l);printf("ERR:%d\n",e);}
  printf("DESC:");out(QSPGetMainDesc());putchar('\n');
  return 0;}
