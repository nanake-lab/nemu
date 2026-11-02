import struct, pathlib, ctypes, os, json
base=pathlib.Path(r'D:\Dev\cache\p4-02\gsv-v5-venv\Lib\site-packages')
handles=[os.add_dll_directory(str(base/'torch'/'lib'))]
def imports(p):
 b=p.read_bytes(); pe=struct.unpack_from('<I',b,60)[0]; n=struct.unpack_from('<H',b,pe+6)[0]; sz=struct.unpack_from('<H',b,pe+20)[0]; opt=pe+24; dd=opt+(112 if struct.unpack_from('<H',b,opt)[0]==523 else 96)
 sections=[struct.unpack_from('<IIII',b,opt+sz+40*i+8) for i in range(n)]
 def off(r):
  for vs,va,rs,rp in sections:
   if va<=r<va+max(vs,rs):return rp+r-va
  return r
 r=struct.unpack_from('<I',b,dd+8)[0]; out=[]
 if not r:return out
 o=off(r)
 while any(b[o:o+20]):
  name=off(struct.unpack_from('<I',b,o+12)[0]); out.append(b[name:b.index(0,name)].decode()); o+=20
 return out
results=[]
for p in sorted((base/'torchcodec').glob('libtorchcodec_core*.dll')):
 deps=[]
 for name in imports(p):
  try: ctypes.WinDLL(name); status='loadable'
  except OSError as e: status=str(e)
  deps.append({'name':name,'load':status})
 results.append({'file':str(p),'imports':deps})
print(json.dumps(results,indent=2))
