import sys,struct
def enc(s):
    u=[]
    for c in s.encode('utf-16-le')[::2] and []: pass
    b=s.encode('utf-16-le'); us=struct.unpack('<%dH'%(len(b)//2),b)
    return [0xFFFB if x==5 else (x-5)&0xFFFF for x in us]
def build(locs):
    u=[]
    def line(v): u.extend(v); u.extend([13,10])
    line([ord(c) for c in "QSPGAME"]); line([ord(c) for c in "5.7.0"]); line(enc(""));line(enc(str(len(locs))))
    for name,code,acts in locs:
        line(enc(name));line(enc(""));line(enc(code.replace("\n","\r\n")));line(enc(str(len(acts))))
        for d,c in acts: line(enc("")); line(enc(d)); line(enc(c.replace("\n","\r\n")))
    return struct.pack('<%dH'%len(u),*u)
