@echo off

cd %~dp0

wasm-pack build --target web

copy /v /y pkg\wasm_0_0_2_bg.wasm /b ..\
copy /v /y pkg\wasm_0_0_2_bg.wasm.d.ts /b ..\
copy /v /y pkg\wasm_0_0_2.d.ts /b ..\
copy /v /y pkg\wasm_0_0_2.js /b ..\
