import subprocess,sys
from gen import build
F=("f","local x\nx=args[0]\nres=x*2",[])
T=[l for l in open('cases59.txt',encoding='utf8').read().split('\n===\n') if l.strip()]
import re
def norm(o):
    out=[]
    for l in o.split('\n'):
        if l.startswith('WIN:'): continue
        if l.startswith('ERR:'): l=l.replace("'s","").replace("'","").rstrip('!').lower()
        out.append(l)
    return '\n'.join(out)
bad=0;known=0
for i,t in enumerate(T):
    d=build([("start",t,[]),F])
    open('t59.qsp','wb').write(d)
    c=subprocess.run(['./cdrv59','t59.qsp'],capture_output=True,text=True,timeout=10)
    r=subprocess.run(['../../target/release/examples/rdrv59','t59.qsp'],capture_output=True,text=True,timeout=10)
    if norm(c.stdout)!=norm(r.stdout):
        if t.startswith('!known'): known+=1; continue
        bad+=1; print("DIFF #%d %r\n  C59: %r\n  RS : %r %r"%(i,t,c.stdout,r.stdout,r.stderr[-200:]))
print("total",len(T),"diffs",bad,"known-differences",known)
sys.exit(1 if bad else 0)
