import { Emu } from "dmg-emu";
import { LitElement, html, css } from "lit-element";

class App extends LitElement {

    static get properties() {
        return {
            play: {type: Boolean, attribute: true},
            cpu: {attribute: false},
            breakpoints: {attribute: false},
            show_debug: {attribute: false}
        }
    }

    static get styles() {
        return css`

            .hider {
                position: absolute;
                right: 5px;
                bottom: 5px;
                padding: 20px;
                background-color: #252731;
                border-radius: 10px;
                cursor: pointer;
            }

            .show {
                right: -20px;
            }

            .debug {
                position: relative;
                padding: 10px 10px;
                width: calc(calc(100vw - 100vh) - 42px);
                height: calc(100vh - 20px);
                background-color: white;
                background-color: #252731;
                color: white;
                overflow-y: scroll;
                margin-left: 0px;
                transition: margin-left .2s ease-out;
            }

            .debug.hide {
                margin-left: calc(-1 * calc(calc(100vw - 100vh) - 42px));
            }
            .container {
                display: flex;
            }

            @media (orientation: portrait) {
                .container {
                    flex-direction: column;
                }  
            }

            .sbs {
                display: flex;
            }
            button {
                border: none;
                background-color: transparent;
                padding: 0px;
                border-right: 10px;
                font-size: 20px;
                cursor: pointer;
            }
            button:active {
                filter: brightness(0.5);
            }
            .wrap {
                background-color: #1D1E23;
                padding: 5px;
                border-radius: 5px;
            }
            .wrap > p {
                margin-top: 0px;
                margin-bottom: 8px;
            }
            p.mid {
                margin-top: 8px;
                margin-bottom: 8px; 
            }

            .debug::-webkit-scrollbar {
                width: 10px;               /* width of the entire scrollbar */
            }

            .debug::-webkit-scrollbar-track {
                background: rgb(37, 39, 49);        /* color of the tracking area */
            }

            .debug::-webkit-scrollbar-thumb {
                background-color: var(--white);    /* color of the scroll thumb */
                border-radius: 20px;       /* roundness of the scroll thumb */
            }
        `
    }
    constructor() {
        super();
        this.mem = [];
        this.show_debug = true;
        
        this.cpu = {
            AF: 0,
            BC: 0,
            DE: 0,
            HL: 0,
            SP: 0,
            PC: 0,
        };
        this.dmg = Emu.new();
    }

    _handleFrame() {
        this.mem = this.dmg.get_mem_state();
        this.cpu = this.dmg.get_cpu_state();
    }

    _handleUpdateMemory() {
        this.mem = this.dmg.get_mem_state();
    }

    formatHex(n) {
        let s = n.toString(16).toUpperCase();
        return s.length === 1 ? `000${s}`
            :  s.length === 2 ? `00${s}`
            :  s.length === 3 ? `0${s}`
            :  s;
    }

    handlePlay() {
        this.play = true;
    }

    handlePause() {
        this.play = false;
    }

    handleTrace() {
        this.shadowRoot.querySelector('dmg-screen').handleStep();
    }

    handleHide() {
        if (this.show_debug){
            this.shadowRoot.querySelector('.debug').classList.toggle('hide');
            setTimeout(() => {
                this.show_debug = !this.show_debug;
            }, 200)
        } else {
            this.show_debug = !this.show_debug;
        }
    }

    _renderVideo() {
        const titles = ["VRAM", "OAM"];
        const elements = [
            html`<vram-debug .play=${this.play} .data=${this.mem}></vram-debug>`, 
            html`<oam-debug .play=${this.play} .data=${this.mem}></oam-debug>`
        ];
        return html`
            <tabbed-card .titles=${titles} .elements=${elements}></tabbed-card>
        `
    }

    _renderIo() {
        const titles = ["Serial", "Joypad", "Timer"];
        const elements = [    
            html`<serial-debug .buffer=${this.dmg.get_serial_buffer()}></serial-debug>`,
            html`<joypad-debug .dmg=${this.dmg} .data=${this.mem}></joypad-debug>`,
            html`<timer-debug .timerIO=${this.dmg.get_timer_state()}></timer-debug>`
        ]
        return html`<tabbed-card .titles=${titles} .elements=${elements}></tabbed-card>`;
    }

    render() {
        return html`
        <div class="hider" @click=${this.handleHide}>🐛</div>
        <div class="container">
            ${ this.show_debug ? html`
            <div class="debug">
                <div class="wrap" style="margin-bottom: 10px;">
                    <button @click="${this.handlePlay}">▶️</button>
                    <button @click="${this.handlePause}">⏸</button>
                    <button @click="${this.handleTrace}">⏭️</button>
                </div>
                <div class="sbs">
                    <div class="wrap"> 
                        <p>Registers</p>
                        <span>AF: ${this.formatHex(this.cpu.AF)}</span><br>
                        <span>BC: ${this.formatHex(this.cpu.BC)}</span><br>
                        <span>DE: ${this.formatHex(this.cpu.DE)}</span><br>
                        <span>HL: ${this.formatHex(this.cpu.HL)}</span><br>
                        <span>SP: ${this.formatHex(this.cpu.SP)}</span><br>
                        <span>PC: ${this.formatHex(this.cpu.PC)}</span><br>
                        <p class="mid">Flags</p>
                        <span>Z: ${this.cpu.AF >> 7 & 1 === 1}</span>
                        <span>N: ${this.cpu.AF >> 6 & 1 === 1}</span><br>
                        <span>C: ${this.cpu.AF >> 5 & 1 === 1}</span>
                        <span>H: ${this.cpu.AF >> 4 & 1 === 1}</span>
                    </div>
                    <program-debug .play=${this.play} .data="${this.mem}" .dmg=${this.dmg} .pc="${this.cpu.PC}"></program-debug>
                </div>
                <memory-debug .play=${this.play} .data="${this.mem}"></memory-debug>
                ${this._renderIo()}
                ${this._renderVideo()}
            </div>
            ` : ''}
            <dmg-screen 
                .play=${this.play} 
                .step=${this.step} 
                .stepping=${this.stepping} 
                @frame=${this._handleFrame}
                @break=${this.handlePause}
                @update-memory=${this._handleUpdateMemory}
                .dmg=${this.dmg}></dmg-screen>
        </div> 
    `
    }

}

customElements.define('dmg-app', App);

