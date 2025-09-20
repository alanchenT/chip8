import init, * as wasm from "./wasm.js"

const WIDTH = 64
const HEIGHT = 32
const SCALE = 15
const TICKS_PER_FRAME = 10

let animFrame = 0

const canvas = document.getElementById("canvas")
canvas.width = WIDTH * SCALE
canvas.height = HEIGHT * SCALE

const input = document.getElementById("fileinput")

const ctx = canvas.getContext("2d")

function clearCanvas() {
    ctx.fillStyle = "black"
    ctx.fillRect(0, 0, canvas.width, canvas.height)
}

clearCanvas()

function mainLoop(emulator) {
    for (let i = 0; i < TICKS_PER_FRAME; i++) {
        emulator.tick()
    }
    emulator.tick_timers()

    clearCanvas()
    ctx.fillStyle = "white"
    emulator.draw_screen(SCALE)

    animFrame = window.requestAnimationFrame(() => {
        mainLoop(emulator)
    })
}

async function run() {
    await init()

    let chip8 = new wasm.EmulatorWasm()

    document.addEventListener("keydown", function (event) {
        chip8.keypress(event, true)
    })

    document.addEventListener("keyup", function (event) {
        chip8.keypress(event, false)
    })

    input.addEventListener("change", function (event) {
        // Stop previous game from rendering
        if (animFrame != 0) {
            window.cancelAnimationFrame(animFrame)
        }

        let file = event.target.files[0]
        if (!file) {
            alert("Failed to open file")
            return
        }

        // Load game as u8 array and send it to wasm to start playing
        let reader = new FileReader()
        reader.onload = function (ev) {
            let buffer = reader.result

            const rom = new Uint8Array(buffer)

            chip8.reset()
            chip8.load_game(rom)

            mainLoop(chip8)
        }
        reader.readAsArrayBuffer(file)

    }, false)
}

run().catch(console.error)