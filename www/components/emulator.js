
import { LitElement, html, css } from 'lit-element';
import { Button } from 'dmg-emu';

let ctx;

class DMGScreen extends LitElement {
    static get properties() {
        return {
            play: {type: Boolean, attribute: false},
            justPaused: {type: Boolean, attribute: false},
            dmg: {attribute: false},
            mem: {attribute: false},
            cpu: {attribute: false},
        }
    }

    static get styles() {
        return css`
            canvas {
                height: 100vh;
                image-rendering: -moz-crisp-edges;
                image-rendering: -webkit-crisp-edges;
                image-rendering: pixelated;
                image-rendering: crisp-edges;
            }
        `
    }

    constructor() {
        super();
        this.play = false;
        this.addEventListener('ondrag', this.handleDrag);
        this.addEventListener('ondrop', this.handleDrop);
        this.addEventListener('keydown', this.handleKeyUp);
        this.addEventListener('keyup', this.handleKeyDown);

        this.buttons = {
            a: false,
            b: false,
            up: false,
            down: false,
            left: false,
            right: false,
            start: false,
            select: false
        };

        if (!this.hasAttribute('tabindex')) {
            this.setAttribute('tabindex', 0);
        }

        this.audioCtx = new AudioContext();
    }

    handleKeyDown(e) {
        e.preventDefault();
        if(e.key === 'a') {
            this.dmg.press_button(Button.A, true);
        } 
        if(e.key === 's') {
            this.dmg.press_button(Button.B, true);
        }

        if(e.key === 'ArrowRight') {
            this.dmg.press_button(Button.Right, true);
        } 
        if(e.key === 'ArrowLeft') {
            this.dmg.press_button(Button.Left, true);
        }
        if(e.key === 'ArrowUp') {
            this.dmg.press_button(Button.Up, true);
        } 
        if(e.key === 'ArrowDown') {
            this.dmg.press_button(Button.Down, true);
        }

        if(e.key === 'Enter') {
            this.dmg.press_button(Button.Start, true);
        }
        if(e.key === 'Shift') {
            this.dmg.press_button(Button.Select, true);
        }
    }

    handleKeyUp(e) {
        e.preventDefault();
        if(e.key === 'a') {
            this.dmg.press_button(Button.A, false);
        } 
        if(e.key === 's') {
            this.dmg.press_button(Button.B, false);
        }

        if(e.key === 'ArrowRight') {
            this.dmg.press_button(Button.Right, false);
        } 
        if(e.key === 'ArrowLeft') {
            this.dmg.press_button(Button.Left, false);
        }
        if(e.key === 'ArrowUp') {
            this.dmg.press_button(Button.Up, false);
        } 
        if(e.key === 'ArrowDown') {
            this.dmg.press_button(Button.Down, false);
        }

        if(e.key === 'Enter') {
            this.dmg.press_button(Button.Start, false);
        }
        if(e.key === 'Shift') {
            this.dmg.press_button(Button.Select, false);
        }
    }

    handleStep() {
        this.dmg.tick();
        this.dispatchEvent((new CustomEvent('frame')));
    }

    handleDrag(event) {
        event.preventDefault();
    }

    emulateAudio() {
        let buffer = this.audioCtx.createBuffer(2, 44100 / 60, 44100);

        var channelR = buffer.getChannelData(0);
        var channelL = buffer.getChannelData(1);
        let data = this.dmg.get_audio_channel1;
        channelR.set(data);
        channelL.set(data);

        var source = this.audioCtx.createBufferSource();
        source.buffer = buffer;
        source.connect(this.audioCtx.destination);
        source.start();
    }

    start() {
        this.dmg.init();

        const tick = () => {
            if (!this.play) {
                if (this.justPaused) {
                    this.dispatchEvent((new CustomEvent('update-memory')));
                }
                this.justPaused = false;
                requestAnimationFrame(tick);
                return;
            }

            let finished_frame = this.dmg.tick_till_frame_done();

            
            if (!finished_frame) {
                this.justPaused = true;
                this.dispatchEvent((new CustomEvent('break')));
            }
            let screen = new Uint8ClampedArray(this.dmg.get_buffer());
            let data = new ImageData(screen, 160, 144);
            this.ctx.putImageData(data, 0, 0);

            //this.emulateAudio();

            this.dispatchEvent((new CustomEvent('frame')));

            this.justPaused = true;
            requestAnimationFrame(tick);
        }
        requestAnimationFrame(tick);
    }

    async handleDrop (event) {
        event.preventDefault();
        let file = event.dataTransfer.items[0].getAsFile();
        let data = await file.arrayBuffer();
        let rom = new Uint8Array(data);
        this.dmg.load_rom_data(rom);
        this.ctx = this.shadowRoot.querySelector('#screen').getContext('2d');
        this.start();
    }

    render() {
        return html`
            <canvas width="160" height="144" @dragover="${this.handleDrag}" @drop="${this.handleDrop}" id="screen"></canvas>
        `
    }
}

customElements.define('dmg-screen', DMGScreen);