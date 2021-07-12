
import { LitElement, html, css } from 'lit-element';
import Model from '../model';
import { Button } from 'dmg-emu';

let ctx;

class DMGScreen extends LitElement {
    static get properties() {
        return {
            play: {type: Boolean, attribute: false},
            justPaused: {type: Boolean, attribute: false},
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
    }

    handleKeyDown(e) {
        console.log(e)
        e.preventDefault();
        if(e.key === 'a') {
            Model.dmg.press_button(Button.A, true);
        } 
        if(e.key === 's') {
            Model.dmg.press_button(Button.B, true);
        }

        if(e.key === 'ArrowRight') {
            Model.dmg.press_button(Button.Right, true);
        } 
        if(e.key === 'ArrowLeft') {
            Model.dmg.press_button(Button.Left, true);
        }
        if(e.key === 'ArrowUp') {
            Model.dmg.press_button(Button.Up, true);
        } 
        if(e.key === 'ArrowDown') {
            Model.dmg.press_button(Button.Down, true);
        }

        if(e.key === 'Enter') {
            Model.dmg.press_button(Button.Start, true);
        }
        if(e.key === 'Shift') {
            Model.dmg.press_button(Button.Select, true);
        }
    }

    handleKeyUp(e) {
        e.preventDefault();
        if(e.key === 'a') {
            Model.dmg.press_button(Button.A, false);
        } 
        if(e.key === 's') {
            Model.dmg.press_button(Button.B, false);
        }

        if(e.key === 'ArrowRight') {
            Model.dmg.press_button(Button.Right, false);
        } 
        if(e.key === 'ArrowLeft') {
            Model.dmg.press_button(Button.Left, false);
        }
        if(e.key === 'ArrowUp') {
            Model.dmg.press_button(Button.Up, false);
        } 
        if(e.key === 'ArrowDown') {
            Model.dmg.press_button(Button.Down, false);
        }

        if(e.key === 'Enter') {
            Model.dmg.press_button(Button.Start, false);
        }
        if(e.key === 'Shift') {
            Model.dmg.press_button(Button.Select, false);
        }
    }

    handleStep() {
        Model.dmg.tick();
        Model.mem = Model.dmg.get_mem_state();
        Model.cpu = Model.dmg.get_cpu_state();
        this.dispatchEvent((new CustomEvent('frame')));
    }

    handleDrag(event) {
        console.log(event);
        event.preventDefault();
    }

    start() {
        Model.dmg.init();

        const tick = () => {

            if (!this.play) {
                if (this.justPaused) {
                    Model.mem = Model.dmg.get_mem_state();
                }
                this.justPaused = false;
                requestAnimationFrame(tick);
                return;
            }

            Model.dmg.tick_till_frame_done();
            let screen = new Uint8ClampedArray(Model.dmg.get_buffer());
            let data = new ImageData(screen, 160, 144);
            this.ctx.putImageData(data, 0, 0);
            this.dispatchEvent((new CustomEvent('frame')));
            Model.cpu = Model.dmg.get_cpu_state();
            Model.mem = Model.dmg.get_mem_state();

            this.justPaused = true;
            requestAnimationFrame(tick);
        }
        requestAnimationFrame(tick);
    }

    async handleDrop (event) {
        event.preventDefault();
        console.log(event);
        let file = event.dataTransfer.items[0].getAsFile();
        let data = await file.arrayBuffer();
        let rom = new Uint8Array(data);
        Model.dmg.load_rom_data(rom);
        console.log(rom);
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