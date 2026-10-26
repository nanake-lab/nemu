@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
if errorlevel 1 exit /b %errorlevel%
"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\cl.exe" /nologo /std:c++17 /EHsc /W4 /WX /O2 /fp:precise /utf-8 /I"D:\Dev\cache\P0-10\source\llama.cpp-b11669\include" /I"D:\Dev\cache\P0-10\source\llama.cpp-b11669\ggml\include" "D:\Dev\nemu-wt\P0-10\spikes\carrier-engine\injection_probe.cpp" /Fe:injection_probe.exe
exit /b %errorlevel%