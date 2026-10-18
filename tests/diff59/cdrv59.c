#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <wchar.h>
#include "bindings/default/qsp_default.h"
static void out(QSPString s){ for(QSP_CHAR*p=s.Str;p<s.End;++p){ if(*p<128) putchar(*p); else printf("\\u%04x",(unsigned)*p);} }
static void msg(QSPString s){ printf("MSG:"); out(s); putchar('\n'); }
static void sw(int t,QSP_BOOL on){printf("WIN:%d:%d\n",t,on);}
int main(int argc,char**argv){
  FILE*f=fopen(argv[1],"rb"); fseek(f,0,SEEK_END); long n=ftell(f); fseek(f,0,SEEK_SET);
  char*b=malloc(n+4); fread(b,1,n,f); fclose(f);
  QSPInit(); QSPSetCallback(QSP_CALL_SHOWMSGSTR,(QSP_CALLBACK)msg);QSPSetCallback(QSP_CALL_SHOWWINDOW,(QSP_CALLBACK)sw);
  if(!QSPLoadGameWorldFromData(b,(int)n,QSP_TRUE)){printf("LOADFAIL\n");return 0;}
  int ok=QSPRestartGame(QSP_TRUE);
  if(!ok){QSPErrorInfo e=QSPGetLastErrorData();printf("ERR:");out(QSPGetErrorDesc(e.ErrorNum));putchar('\n');}
  printf("DESC:");out(QSPGetMainDesc());putchar('\n');
  return 0;}
