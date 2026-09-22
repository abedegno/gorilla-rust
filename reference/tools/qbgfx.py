"""The QBasic drawing primitives as measured from the original."""
import math
W,H=640,350
ASPECT=0.73

def rha(v):
    """Round half away from zero. CIRCLE uses this to scale an axis.

    Measured on a radius 40 circle at aspect 0.5, which is the only case in
    the fixtures that produces exact half values."""
    return math.floor(v+0.5) if v>=0 else math.ceil(v-0.5)

def cint(v):
    """QBasic's CINT, rounding half to even.

    Every coordinate argument goes through this: LINE, PUT, POINT, and a
    CIRCLE centre and radius. Measured with ROUNDING.BAS, where PUT at
    150.5 lands on 150 and POINT at 499.5 reads pixel 500, both even."""
    return int(round(v))

class Screen:
    def __init__(s,w=W,h=H): s.w,s.h=w,h; s.p=bytearray(w*h)
    def pset(s,x,y,c):
        if 0<=x<s.w and 0<=y<s.h: s.p[y*s.w+x]=c
    def point(s,x,y):
        return s.p[y*s.w+x] if 0<=x<s.w and 0<=y<s.h else 0

    def _clip(s,x1,y1,x2,y2):
        w,h=s.w-1,s.h-1
        if 0<=x1<=w and 0<=y1<=h and 0<=x2<=w and 0<=y2<=h: return (x1,y1,x2,y2)
        dx,dy=float(x2-x1),float(y2-y1); t0,t1=0.0,1.0
        for p,q in ((-dx,float(x1)),(dx,float(w-x1)),(-dy,float(y1)),(dy,float(h-y1))):
            if p==0.0:
                if q<0: return None
            else:
                r=q/p
                if p<0:
                    if r>t1: return None
                    if r>t0: t0=r
                else:
                    if r<t0: return None
                    if r<t1: t1=r
        f=lambda t:(rha(x1+t*dx), rha(y1+t*dy))
        a,b=f(t0),f(t1); return (a[0],a[1],b[0],b[1])

    def line(s,x1,y1,x2,y2,c):
        cl=s._clip(x1,y1,x2,y2)
        if cl is None: return
        x1,y1,x2,y2=cl
        dx,dy=x2-x1,y2-y1; adx,ady=abs(dx),abs(dy)
        if adx==0 and ady==0: s.pset(x1,y1,c); return
        sg=lambda v:(v>0)-(v<0)
        mx = adx>=ady
        dmaj,dmin = (adx,ady) if mx else (ady,adx)
        smaj,smin = (sg(dx),sg(dy)) if mx else (sg(dy),sg(dx))
        b=3*dmaj//4
        for k in range(dmaj+1):
            m=smin*((k*dmin+b)//dmaj)
            if mx: s.pset(x1+smaj*k, y1+m, c)
            else:  s.pset(x1+m, y1+smaj*k, c)

    def line_box(s,x1,y1,x2,y2,c):
        s.line(x1,y1,x2,y1,c); s.line(x2,y1,x2,y2,c)
        s.line(x2,y2,x1,y2,c); s.line(x1,y2,x1,y1,c)

    def line_fill(s,x1,y1,x2,y2,c):
        for y in range(max(0,min(y1,y2)), min(s.h-1,max(y1,y2))+1):
            base=y*s.w
            for x in range(max(0,min(x1,x2)), min(s.w-1,max(x1,x2))+1): s.p[base+x]=c

    def circle(s,cx,cy,r,c,start=None,end=None,aspect=None):
        a=ASPECT if aspect is None else aspect
        icx,icy,R=cint(cx),cint(cy),cint(r)
        if R==0: s.pset(icx,icy,c); return
        def place(dx,dy):
            if a<0:
                k=1.0-(abs(a)-math.floor(abs(a))); return (icx+dx, icy+rha(dy*k))
            if a>1.0: return (icx+rha(dx/a), icy+dy)
            return (icx+dx, icy+rha(dy*a))
        P=[]; x,y,d=0,R,3-2*R
        while x<=y:
            P += [(x,y),(-x,y),(x,-y),(-x,-y),(y,x),(-y,x),(y,-x),(-y,-x)]
            if d<0: d+=4*x+6
            else: d+=4*(x-y)+10; y-=1
            x+=1
        TAU=2*math.pi
        if start is None and end is None:
            for p in P: s.pset(*place(*p), c); 
            return
        st=(start or 0.0)%TAU; en=(end if end is not None else TAU)%TAU
        if en<=st: en+=TAU
        for dx,dy in P:
            ang=math.atan2(-dy,dx)%TAU
            if (st<=ang<=en) or (st<=ang+TAU<=en): s.pset(*place(dx,dy), c)
        for ang in (st,en):
            s.pset(icx+cint(r*math.cos(ang)), icy+cint(-r*math.sin(ang)*a), c)

    def paint(s,sx,sy,fill,border):
        def blk(x,y):
            v=s.point(x,y); return v==border or v==fill
        if not(0<=sx<s.w and 0<=sy<s.h) or blk(sx,sy): return
        stk=[(sx,sy)]
        while stk:
            x,y=stk.pop()
            if blk(x,y): continue
            l=x
            while l>0 and not blk(l-1,y): l-=1
            r=x
            while r<s.w-1 and not blk(r+1,y): r+=1
            base=y*s.w
            for cx in range(l,r+1): s.p[base+cx]=fill
            for ny in (y-1,y+1):
                if 0<=ny<s.h:
                    run=False
                    for cx in range(l,r+1):
                        ok=not blk(cx,ny)
                        if ok and not run: stk.append((cx,ny))
                        run=ok

    def get(s,x1,y1,x2,y2):
        lo_x,hi_x=min(x1,x2),max(x1,x2); lo_y,hi_y=min(y1,y2),max(y1,y2)
        w,h=hi_x-lo_x+1, hi_y-lo_y+1
        return (w,h,[s.point(x,y) for y in range(lo_y,hi_y+1) for x in range(lo_x,hi_x+1)])

    def put(s,x,y,sp,mode):
        w,h,px=sp
        for sy in range(h):
            for sx in range(w):
                dx,dy=x+sx,y+sy
                if not(0<=dx<s.w and 0<=dy<s.h): continue
                v=px[sy*w+sx]; i=dy*s.w+dx
                s.p[i] = v if mode=='pset' else (s.p[i]^v)

def xy(v):
    'Convert a fractional coordinate the way QBasic does.'
    return cint(v)

def diff(scr,path):
    fb=open(path,'rb').read()
    return sum(1 for i in range(len(fb)) if scr.p[i]!=fb[i])
