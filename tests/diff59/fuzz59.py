import random,subprocess,sys
from gen import build
random.seed(int(sys.argv[1]) if len(sys.argv)>1 else 1)
N=int(sys.argv[2]) if len(sys.argv)>2 else 300
W=['pear','Apple','mango','kiwi','fig','date','plum','lime','x','zeta','Éclair','яблоко']
def prog():
    L=[];ty=random.choice(['n','s']);k=random.randint(0,6)
    vals=random.sample(range(-20,50),k) if ty=='n' else random.sample(W,min(k,len(W)))
    for i,v in enumerate(vals):
        L.append(("a[%d]=%d"%(i,v)) if ty=='n' else "$a[%d]='%s'"%(i,v))
    nm='a' if ty=='n' else '$a'
    for _ in range(random.randint(1,3)):
        c=random.random()
        if c<.4: L.append("sortarr '%s'%s"%(nm,random.choice(['',', 0',', 1'])))
        elif c<.6: L.append("scanstr 'r','%s','%s'%s"%(' '.join(random.choices(W,k=3)),random.choice(['\\w+','[aeiou]','(\\w)(\\w)','\\s','x*']),random.choice(['',',1',',2',',0'])))
        elif c<.8: L.append("setvar '%s', %s%s"%(nm,random.randint(0,9) if ty=='n' else "'q%d'"%random.randint(0,9),random.choice(['',', %d'%random.randint(0,max(k-1,0))])))
        else: L.append("msg arritem('%s'%s)"%(nm,random.choice(['',', %d'%random.randint(0,max(k-1,0))])))
    L.append("msg arrsize('a')")
    L.append("i=0\n:la\nif i<arrsize('a'): msg %sa[i] & i=i+1 & jump 'la'"%('$' if ty=='s' else ''))
    L.append("msg arrsize('r')")
    L.append("i=0\n:lr\nif i<arrsize('r'): msg $r[i] & i=i+1 & jump 'lr'")
    return '\n'.join(L)
bad=0
for i in range(N):
    t=prog(); open('f59.qsp','wb').write(build([("start",t,[]),("f","",[])]))
    c=subprocess.run(['./cdrv59','f59.qsp'],capture_output=True,text=True).stdout
    r=subprocess.run(['../../target/release/examples/rdrv59','f59.qsp'],capture_output=True,text=True).stdout
    f=lambda o:'\n'.join(l.replace("'s","").replace("'","").lower() if l.startswith('ERR:') else l for l in o.split('\n') if not l.startswith('WIN:'))
    if f(c)!=f(r): bad+=1; print("DIFF",repr(t),'\n',repr(f(c)),'\n',repr(f(r))) if bad<4 else 0
print("fuzz",N,"diffs",bad)
