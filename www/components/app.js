import { LitElement, html, css } from "lit-element";
import Model from '../model.js';

class App extends LitElement {

    static get properties() {
        return {
            play: {type: Boolean, attribute: true},
        }
    }

    static get styles() {
        return css`
            .debug {
                padding: 10px 10px;
                width: calc(calc(100vw - 100vh) - 42px);
                height: calc(100vh - 20px);
                background-color: white;
                background-color: #252731;
                color: white;
                overflow-y: scroll;
            }
            .container {
                display: flex;
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

    _handleFrame() {
        this.requestUpdate();
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

    render() {
        return html`
        <div class="container">
            <div class="debug">
                <div class="wrap" style="margin-bottom: 10px;">
                    <button @click="${this.handlePlay}">▶️</button>
                    <button @click="${this.handlePause}">⏸</button>
                    <button @click="${this.handleTrace}">⏭️</button>
                </div>
                <div class="sbs">
                    <div class="wrap"> 
                        <p>Registers</p>
                        <span>AF: ${this.formatHex(Model.cpu.AF)}</span><br>
                        <span>BC: ${this.formatHex(Model.cpu.BC)}</span><br>
                        <span>DE: ${this.formatHex(Model.cpu.DE)}</span><br>
                        <span>HL: ${this.formatHex(Model.cpu.HL)}</span><br>
                        <span>SP: ${this.formatHex(Model.cpu.SP)}</span><br>
                        <span>PC: ${this.formatHex(Model.cpu.PC)}</span><br>
                        <p class="mid">Flags</p>
                        <span>Z: ${Model.cpu.AF >> 7 & 1 === 1}</span>
                        <span>N: ${Model.cpu.AF >> 6 & 1 === 1}</span><br>
                        <span>C: ${Model.cpu.AF >> 5 & 1 === 1}</span>
                        <span>H: ${Model.cpu.AF >> 4 & 1 === 1}</span>
                    </div>
                    <program-debug .play=${this.play} .data="${Model.mem}" .pc="${Model.cpu.PC}"></program-debug>
                </div>
                <memory-debug .play=${this.play} .data="${Model.mem}"></memory-debug>
                <vram-debug .play=${this.play} .data=${Model.mem}></vram-debug>
                <oam-debug .data=${Model.mem}></oam-debug>
                <joypad-debug .data=${Model.mem}></joypad-debug>
            </div>
            <dmg-screen .play=${this.play} .step=${this.step} .stepping=${this.stepping} @frame=${this._handleFrame}></dmg-screen>
        </div> 
    `
    }

}

customElements.define('dmg-app', App);

