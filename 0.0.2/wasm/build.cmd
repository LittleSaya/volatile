@echo off

cd %~dp0

set PROFILE=--release

if "%1" == "dev" (
    set PROFILE=--dev
)

wasm-pack build %PROFILE% --target web

copy /v /y pkg\wasm_0_0_2_bg.wasm /b ..\
copy /v /y pkg\wasm_0_0_2_bg.wasm.d.ts /b ..\
copy /v /y pkg\wasm_0_0_2.d.ts /b ..\
copy /v /y pkg\wasm_0_0_2.js /b ..\
