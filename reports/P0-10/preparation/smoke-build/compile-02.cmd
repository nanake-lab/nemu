@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
if errorlevel 1 exit /b %errorlevel%
cl.exe /nologo /std:c++17 /EHsc /W4 /WX /O2 /utf-8 /I"D:\Dev\cache\P0-10\source\llama.cpp-b11669\include" /I"D:\Dev\cache\P0-10\source\llama.cpp-b11669\ggml\include" "D:\Dev\nemu-wt\P0-10\spikes\carrier-engine\smoke.cpp" /Fe:smoke.exe
exit /b %errorlevel%