"""Builds crates/gl-store/data/region.json from the World Currencies workbook (sheets Countries and World Currencies)."""
import openpyxl,re,json,sys,collections
# Usage: python3 tools/gen_region.py <World_Currencies workbook.xlsx> crates/gl-store/data/region.json
SRC,OUT=sys.argv[1],sys.argv[2]
wb=openpyxl.load_workbook(SRC,data_only=True)
rows=[r for r in list(wb['Countries'].iter_rows(values_only=True))[1:] if r[0]]
wc=[r for r in list(wb['World Currencies'].iter_rows(values_only=True))[1:] if r[0]]
NUM=r"\d[\d.,' ]*\d|\d"
def tpl(s,sym):
    m=re.search(NUM,s); t=s.replace(m.group(0),'[n]') if m else s
    return t.replace(sym,'[s]'),(m.group(0) if m else '')
PAT={('[n] [s]','-[n] [s]'):'number_space_symbol',('[s][n]','-[s][n]'):'symbol_number',('[s] [n]','-[s] [n]'):'symbol_space_number',
 ('[s] [n]','[s] -[n]'):'symbol_space_minus',('[s] [n]','[s]-[n]'):'symbol_space_minus_tight',('[s][n]','[s]-[n]'):'symbol_minus_tight',
 ('[n][s]','-[n][s]'):'number_symbol',('[n][s]00','-[n][s]00'):'cifrao'}
def number_fmt_decimal(n): return n[-3] if len(n)>3 and n[-3] in '.,' else '.'
def thou(n):
    t=[c for c in n[:-3] if not c.isdigit()]
    return t[0] if t else ''
def frac_len(pos,n):
    m=re.search(NUM,pos); s=m.group(0)
    d=number_fmt_decimal(n)
    # integer part of 1 million has groups of exactly 3 digits after separators; a trailing group with !=3 digits or a 3-digit group following the decimal char is ambiguous
    parts=s.split(d)
    if len(parts)==2 and not (len(parts[1])==3 and thou(n)==d): return len(parts[1])
    return 0
def short_date(ex):
    s=ex.replace('2026','\x01')
    s=s.replace('23','dd').replace('11','MM').replace('26','yy').replace('\x01','yyyy')
    assert all(x in s for x in ('dd','MM','yy')),ex
    return s
WORD=re.compile(r"[^\s\d.,;:/\-–()'“”]+")
def esc(t): return t.replace("'","''")
def long_date(ex):
    s=ex
    y=s.find('2026'); assert y>=0,ex
    s=s.replace('2026','\x01',1)
    s=s.replace('23','\x02',1)
    words=[(m.start(),m.group(0)) for m in WORD.finditer(s) if not re.fullmatch(r'[\x01\x02]+',m.group(0))]
    # tokens like \x01 attached to letters (e.g. 23日): handle by splitting around markers
    parts=re.split(r'([\x01\x02])',s); out=[]; month_done=False
    # month = longest word-run among the non-marker parts, unless a numeric month '11' is present
    has_num_month='11' in s
    longest=None
    if not has_num_month:
        cands=[w for p in parts if p not in('\x01','\x02') for w in WORD.findall(p)]
        longest=max(cands,key=len) if cands else None
    for p in parts:
        if p=='\x01': out.append('yyyy'); continue
        if p=='\x02': out.append('d'); continue
        if has_num_month and '11' in p and not month_done:
            a,b=p.split('11',1); out.append(esc_text(a)); out.append('M'); out.append(esc_text(b)); month_done=True; continue
        pos=0; seg=''
        for m in WORD.finditer(p):
            seg+=esc_plain(p[pos:m.start()])
            w=m.group(0)
            if longest and w==longest and not month_done: seg+='MMMM'; month_done=True
            else: seg+="'"+w.replace("'","''")+"'"
            pos=m.end()
        seg+=esc_plain(p[pos:]); out.append(seg)
    r=''.join(out)
    r=r.replace("''","''")
    return r
def esc_plain(t): return t.replace("'","''") if "'" in t else t
def esc_text(t):
    return esc_plain(t) if not WORD.search(t) else ''.join(("'"+w.replace("'","''")+"'") if WORD.fullmatch(w) else esc_plain(w) for w in re.split(r'(\s+|[.,;:/\-–()])',t) if w)
def short_time(code): return code
def long_time(code):
    i=code.find('mm')
    sep='.' if i>0 and code[i-1]=='.' else ':'
    return code[:i+2]+sep+'ss'+code[i+2:]
countries=[]
for r in rows:
    name,c2,c3,cname,ccode,sym,num,pos,neg,pi,ni,sd,ld,clock,tcode=r[:15]
    p,_=tpl(pos,sym); n,_=tpl(neg,sym)
    pat=PAT[(p,n)]
    dec=number_fmt_decimal(num); th=thou(num)
    indian=bool(re.fullmatch(r'\d\d,\d\d,\d{3}[.,]\d\d',num))
    if pat=='cifrao': cd=2
    else: cd=frac_len(pos,num)
    countries.append(dict(code=c2,name=name,currency=ccode,symbol=sym,decimal=dec,thousands=th,indian=indian,cur_decimals=cd,pattern=pat,short_date=short_date(sd),long_date=long_date(ld),short_time=short_time(tcode),long_time=long_time(tcode),locale=r[25]))
countries.sort(key=lambda c:c['name'])
curs=[]
for r in wc:
    nm,iso,sym=r[0],r[1],r[2]
    pos=r[4]; m=re.search(NUM,pos); s=m.group(0)
    # decimals: digits after the last separator when that group is not exactly the 3-digit thousands group of "1 000 000"
    mm=re.fullmatch(r"1([.,' ])000([.,' ])000(?:([.,])(\d+))?",s)
    d=len(mm.group(4)) if mm and mm.group(4) else 0
    curs.append(dict(code=iso,name=nm,symbol=sym,decimals=d))
json.dump(dict(countries=countries,currencies=curs),open(OUT,'w',encoding='utf-8'),ensure_ascii=False,indent=0)
print(len(countries),'countries,',len(curs),'currencies')
