# chip8
A CHIP-8 emulator. Originally conceived by the dead, now reconceived by the soon-to-be dead.


## Usage
You may run the emulator with either the terminal or a browser. 

To run with the terminal, make sure SDL2 is properly installed. Good luck with that. Actually, scratch that. I was so kind as to provide you the `.dll` and `.lib` files necessary for compilation.

To run with a browser, first `cd` into `/web` and run 
```
$ wasm-pack build --target web
```
Oh, you might have to install `wasm-pack` first. I don't know. Figure it out yourself.

You should now see a `/pkg` directory in `/web`. Move the files named `wasm_bg.wasm` and `wasm.js` into `/web`. Why do you have to do this? I don't know. Figure it out yourself.

Now, you can host a local server and open `/web/index.html` without anything bricking up. Upload some ROM and enjoy your 15 seconds of fun, like they did in the old days.