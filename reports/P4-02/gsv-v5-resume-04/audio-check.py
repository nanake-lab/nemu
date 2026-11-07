import os, pathlib, json, torch, torchaudio
root=pathlib.Path(__file__).parent
binpath=next((root/'ffmpeg').glob('*/bin'))
handle=os.add_dll_directory(str(binpath))
for lang in ('zh','en'):
 a,sr=torchaudio.load(str(pathlib.Path(r'D:\Dev\cache\p4-02\gsv-v5-test-voice')/(lang+'.wav')))
 print(json.dumps({'language':lang,'rate':sr,'shape':list(a.shape),'cuda_initialized':torch.cuda.is_initialized()}),flush=True)
